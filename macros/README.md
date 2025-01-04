# Overview
This is an attribute macro that implements the `Into<Value>` trait for
annotated structs and enums to be used with [Handybars](https://github.com/0x00002a/handybars). Please refer
to the main [Handybars crate](https://github.com/0x00002a/handybars) for information on how to use!

# Implementation Notes
Annotating an enum or a struct with `#[handybars_value]` generates `Into<Value>` implementations
for the item. For example, the `#[handybars_value]` attribute on the enum:
```rust
#[handybars_value]
enum SimpleEnumProp {
    A,
    B,
}
```
... will result in the following code being generated for the `SimpleEnumProp`:
```rust
impl<'v> Into<handybars::Value<'v>> for SimpleEnumProp {
    fn into(self) -> handybars::Value<'v> {
        match self {
            SimpleEnumProp::A => handybars::Value::String(std::borrow::Cow::from("A")),
            SimpleEnumProp::B => handybars::Value::String(std::borrow::Cow::from("B")),
        }
    }
}
```

## Why use an attribute and not a derive process macro?

Derive Macros do not support implementing traits with generic arguments. In this case we
need to implement `Into<Value>` for the annotated enum or struct. If `Value` had been a
trait and not an enum, a derive macro would have been appropriate.
