use std::{borrow::Cow, collections::BTreeMap};

/// Object value with 0 or more properties
#[derive(Clone, Debug, Default, PartialEq, Eq, Hash)]
pub struct Object<'a> {
    pub(crate) values: BTreeMap<Cow<'a, str>, Value<'a>>,
}

impl<'a> Object<'a> {
    /// Construct a new object with no properties
    pub fn new() -> Self {
        Self::default()
    }
    /// Add a property to an object
    ///
    /// Panics: if name contains a `.`
    ///
    /// ```should_panic
    /// # use handybars::Object;
    /// let mut obj = Object::new();
    /// obj.add_property("a.b", "c"); // boom
    /// ```
    pub fn add_property(
        &mut self,
        name: impl Into<Cow<'a, str>>,
        value: impl Into<Value<'a>>,
    ) -> &mut Self {
        let name = name.into();
        assert!(!name.contains('.'), "property name may not contain dots");
        self.values.insert(name, value.into());
        self
    }
    /// Add a property with builder syntax
    ///
    /// Panics: if name contains a '.'
    pub fn with_property(
        mut self,
        name: impl Into<Cow<'a, str>>,
        value: impl Into<Value<'a>>,
    ) -> Self {
        self.add_property(name, value);
        self
    }
    /// Get previousely set property
    pub fn property(&self, name: &str) -> Option<&Value<'a>> {
        self.values.get(name)
    }
}

/// Value that variables can be expanded to
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum Value<'a> {
    /// Simple string substitution
    String(Cow<'a, str>),
    /// Object with additional level of path
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
    #[allow(missing_docs)]
    #[must_use]
    pub fn as_object(&self) -> Option<&Object<'a>> {
        if let Self::Object(v) = self {
            Some(v)
        } else {
            None
        }
    }

    #[allow(missing_docs)]
    #[must_use]
    pub fn as_object_mut(&mut self) -> Option<&mut Object<'a>> {
        if let Self::Object(v) = self {
            Some(v)
        } else {
            None
        }
    }

    #[allow(missing_docs)]
    #[must_use]
    pub fn as_string(&self) -> Option<&Cow<'a, str>> {
        if let Self::String(v) = self {
            Some(v)
        } else {
            None
        }
    }

    /// Returns `true` if the value is [`Object`].
    ///
    /// [`Object`]: Value::Object
    #[must_use]
    pub fn is_object(&self) -> bool {
        matches!(self, Self::Object(..))
    }

    /// Returns `true` if the value is [`String`].
    ///
    /// [`String`]: Value::String
    #[must_use]
    pub fn is_string(&self) -> bool {
        matches!(self, Self::String(..))
    }
}
impl From<String> for Value<'static> {
    fn from(value: String) -> Self {
        Self::String(Cow::Owned(value))
    }
}
