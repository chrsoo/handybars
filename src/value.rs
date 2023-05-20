use std::{borrow::Cow, collections::BTreeMap};

#[derive(Clone, Debug, Default)]
pub struct Object<'a> {
    pub(crate) values: BTreeMap<Cow<'a, str>, Value<'a>>,
}

impl<'a> Object<'a> {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn add_property(
        &mut self,
        name: impl Into<Cow<'a, str>>,
        value: impl Into<Value<'a>>,
    ) -> &mut Self {
        self.values.insert(name.into(), value.into());
        self
    }
    pub fn with_property(
        mut self,
        name: impl Into<Cow<'a, str>>,
        value: impl Into<Value<'a>>,
    ) -> Self {
        self.add_property(name, value);
        self
    }
}

#[derive(Clone, Debug)]
pub enum Value<'a> {
    String(Cow<'a, str>),
    Object(Object<'a>),
}
impl<'a> From<Object<'a>> for Value<'a> {
    fn from(value: Object<'a>) -> Self {
        Self::Object(value)
    }
}
impl<'a> From<Cow<'a, str>> for Value<'a> {
    fn from(value: Cow<'a, str>) -> Self {
        Self::String(value)
    }
}
impl<'a> From<&'a str> for Value<'a> {
    fn from(value: &'a str) -> Self {
        Self::String(value.into())
    }
}
impl<'a> Value<'a> {
    pub fn to_cow_str(&self) -> Cow<'a, str> {
        match self {
            Value::String(s) => s.clone(),
            Value::Object(_o) => todo!(),
        }
    }
}
