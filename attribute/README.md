# Handybars Attribute

This is an attribute macro that implements the `Into<Value>` trait for
annotated structs and enums used with [Handybars](https://github.com/0x00002a/handybars).

## Usage
Include the handybars_attribute depenendcy in your `Cargo.toml` dependencies:
```toml
[dependencies]
handybars = "0.2"
handybars_attribute = "0.2"
```
Import the handybars and the macro in your rust code:
```rust
use handybars::{Context, Variable};
use handybars_attribute::handybars_value;
```
Use simple enums as [Value::String](https://docs.rs/handybars/latest/handybars/enum.Value.html) variants:
```rust
#[handybars_value]
enum SimpleEnumProp {
    A,
    B,
}
```
Use structs as [Value::Object](https://docs.rs/handybars/latest/handybars/enum.Value.html) variants::
```rust
#[handybars_value]
struct StructVal<'a> {
    field_1: u16,
    field_2: String,
    field_3: &'a str,
    field_4: SimpleEnumProp,
}
```
Combine enums and structs into more complex objects:
```rust
#[handybars_value]
struct TestObject<'a> {
    prop_0: String,
    prop_1: u64,
    prop_2: &'a str,
    prop_3: StructVal<'a>,
    prop_4: SimpleEnumProp,
}
```
Example on using the above enums and structs:
```rust
let v = TestObject {
    prop_0: "p0_val".to_owned(),
    prop_1: 1,
    prop_2: "p2_val",
    prop_3: StructVal {
        field_1: 30,
        field_2: "f32_val".to_owned(),
        field_3: "f33_val",
        field_4: SimpleEnumProp::A,
    },
    prop_4: SimpleEnumProp::B,
};
let c = Context::new().with_define(Variable::single("obj"), v);
assert_eq!("1", c.render("{{ obj.prop_1 }}").unwrap());
assert_eq!("A", c.render("{{ obj.prop_3.field_4 }}").unwrap());
assert_eq!("f33_val", c.render("{{ obj.prop_3.field_3 }}").unwrap());
```
The running code for the above can be found as a [macro test case](tests/handybars_macro.rs).

> [!WARNING]
> Enums with variant values are currently **not supported**. Enum with variants like the following will not compile:
```rust
#[handybar_value]
enum ComplexEnumProp<'a> {
    Var1(SimpleEnumProp),
    Var2(String),
    Var3(StructVal<'a>),
}
```
## Implementation
Annotating an enum or a struct with `#[handybars_value]` generates `Into<Value>` implementations
for the item.

For example, the following enum...
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