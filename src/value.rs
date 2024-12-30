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
impl<'a,T> From<Option<T>> for Value<'a>
where T: Into<Value<'a>> {
    fn from(value: Option<T>) -> Self {
        if let Some(v) = value {
            v.into()
        } else {
            Self::String("".into())
        }
    }
}
macro_rules! value_from_num {
    ($typ:ident) => {
        impl<'a> From<$typ> for Value<'a> {
            fn from(value: $typ) -> Self {
                Value::String(value.to_string().into())
            }
        }
    };
}
value_from_num!(i8);
value_from_num!(i16);
value_from_num!(i32);
value_from_num!(i64);
value_from_num!(i128);
value_from_num!(isize);

value_from_num!(u8);
value_from_num!(u16);
value_from_num!(u32);
value_from_num!(u64);
value_from_num!(u128);
value_from_num!(usize);

value_from_num!(f32);
value_from_num!(f64);

value_from_num!(char);
value_from_num!(bool);

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

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::Value;

    #[test]
    fn value_from_hex() {
        assert_eq!(Value::String(Cow::from("42")), From::from(0x0000002a));
    }

    #[test]
    fn value_from_option() {
        let t: Option<&str> = None;
        assert_eq!(Value::String(Cow::from("")), From::from(t));
        assert_eq!(Value::String(Cow::from("42")), From::from(Option::Some("42")));
        assert_eq!(Value::String(Cow::from("42")), From::from(Option::Some(42)));
    }

    #[test]
    fn value_from_unsigned_int() {
        assert_eq!(Value::String(Cow::from("42")), From::from(42u8));
        assert_eq!(Value::String(Cow::from("42")), From::from(42u16));
        assert_eq!(Value::String(Cow::from("42")), From::from(42u32));
        assert_eq!(Value::String(Cow::from("42")), From::from(42u64));
        assert_eq!(Value::String(Cow::from("42")), From::from(42u128));
    }

    #[test]
    fn value_from_signed_int() {
        assert_eq!(Value::String(Cow::from("-42")), From::from(-42i8));
        assert_eq!(Value::String(Cow::from("-42")), From::from(-42i16));
        assert_eq!(Value::String(Cow::from("-42")), From::from(-42i32));
        assert_eq!(Value::String(Cow::from("-42")), From::from(-42i64));
        assert_eq!(Value::String(Cow::from("-42")), From::from(-42i128));
    }

    #[test]
    fn value_from_bool() {
        assert_eq!(Value::String(Cow::from("true")), From::from(true));
        assert_eq!(Value::String(Cow::from("false")), From::from(false));
    }

    #[test]
    fn value_from_ptr() {
        assert_eq!(Value::String(Cow::from("42")), From::from(42usize));
        assert_eq!(Value::String(Cow::from("42")), From::from(42isize));
    }

    #[test]
    fn value_from_float() {
        assert_eq!(Value::String(Cow::from("42.242")), From::from(42.242f32));
        assert_eq!(Value::String(Cow::from("42.242")), From::from(42.242f64));
    }

    #[test]
    fn value_from_char() {
        assert_eq!(Value::String(Cow::from("*")), From::from('*'));
    }

}