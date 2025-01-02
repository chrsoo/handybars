use proc_macro::{Span, TokenStream};
use quote::quote;
use syn::{GenericParam, Ident, Lifetime, LifetimeParam};

#[proc_macro_attribute]
pub fn handybars_value(attr: TokenStream, item: TokenStream) -> TokenStream {
    assert!(
        attr.is_empty(),
        "the handybar_value macro does not take arguments"
    );
    assert!(
        !item.is_empty(),
        "the handybar_value macro must be applied to a struct or an enum"
    );

    // let ast: syn::DeriveInput = syn::parse(item).unwrap();
    let ast: syn::Item = syn::parse(item).unwrap();

    let gen = match ast {
        syn::Item::Enum(item) => {
            let fields = item
                .variants
                .iter()
                .map(|field| &field.ident)
                .collect::<Vec<&Ident>>();
            // let name = format_ident!("{}", item_enum.ident);
            let name = &item.ident;

            let mut gen_clone = item.generics.clone();
            let lt = if let Some(lt) = gen_clone.lifetimes().next() {
                lt
            } else {
                let lt = Lifetime::new("'v", Span::call_site().into());
                let ltp = LifetimeParam::new(lt);
                gen_clone.params.push(GenericParam::from(ltp));
                gen_clone.lifetimes().last().unwrap()
            };
            let (impl_gen, _, _) = gen_clone.split_for_impl();
            let (_, type_gen, where_clause) = item.generics.split_for_impl();

            quote! {
                impl #impl_gen Into<handybars::Value<#lt>> for #name #type_gen #where_clause {
                // impl<'v> Into<handybars::Value<'v>> for #name {
                    fn into(self) -> handybars::Value<#lt> {
                        match self {
                        #(
                            #name::#fields => handybars::Value::String(std::borrow::Cow::from(stringify!(#fields))),
                        )*
                        }
                    }
                }
                #item
            }
        }
        syn::Item::Struct(item) => {
            let fields = item
                .fields
                .iter()
                .map(|field| -> &Ident { field.ident.as_ref().unwrap() });
            let name = &item.ident;

            let mut gen_clone = item.generics.clone();
            let lt = if let Some(lt) = gen_clone.lifetimes().next() {
                lt
            } else {
                let lt = Lifetime::new("'v", Span::call_site().into());
                let ltp = LifetimeParam::new(lt);
                gen_clone.params.push(GenericParam::from(ltp));
                gen_clone.lifetimes().last().unwrap()
            };
            let (impl_gen, _, _) = gen_clone.split_for_impl();
            let (_, type_gen, where_clause) = item.generics.split_for_impl();

            quote! {
                impl #impl_gen Into<handybars::Value<#lt>> for #name #type_gen #where_clause {
                // impl<'v> Into<handybars::Value<'v>> for #name {
                    fn into(self) -> handybars::Value<#lt> {
                        let mut obj = handybars::Object::new();
                        #(
                            obj.add_property(stringify!(#fields), Into::<handybars::Value>::into(self.#fields));
                        )*
                        handybars::Value::Object(obj)
                    }
                }
                #item
            }
        }
        _ => panic!("Handybar `value` macro only supports enum and struct items"),
        // _ => ast.into_token_stream(),
    };
    gen.into()
}
