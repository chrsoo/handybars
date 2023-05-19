use std::{borrow::Cow, str::FromStr};

mod parse;
mod value;

type VariableEl = Cow<'static, str>;

enum VariableInner {
    Segments(Vec<VariableEl>),
    Single(VariableEl),
}

#[repr(transparent)]
pub struct Variable {
    inner: VariableInner,
}

impl Variable {
    pub fn single_unchecked(name: impl Into<VariableEl>) -> Self {
        Self {
            inner: VariableInner::Single(name.into()),
        }
    }
    pub fn single(var: impl Into<VariableEl>) -> Self {
        let val = var.into();
        assert!(
            !val.contains('.'),
            "single cannot contain dot separator. Use parse if you want that"
        );
        Self::single_unchecked(val)
    }
    pub fn from_parts(parts: impl IntoIterator<Item = impl Into<VariableEl>>) -> Self {
        Self {
            inner: VariableInner::Segments(parts.into_iter().map(|p| p.into()).collect()),
        }
    }
    pub fn from_string(s: &str) -> Result<Self, <Self as FromStr>::Err> {
        s.parse()
    }
}

pub struct VariableParseError {
    offset: usize,
}

impl FromStr for Variable {
    type Err = VariableParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let segments = s.split(".").collect::<Vec<_>>();
        let mut offset = 0;
        for seg in &segments {
            if seg.is_empty() {
                return Err(VariableParseError { offset });
            }
            offset += seg.len() + 1;
        }
        Ok(Self::from_parts(segments))
    }
}
