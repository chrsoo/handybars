use std::{borrow::Cow, str::FromStr};

mod parse;
mod value;

type VariableEl<'a> = Cow<'a, str>;

#[derive(Debug, PartialEq, Eq)]
enum VariableInner<'a> {
    Segments(Vec<VariableEl<'a>>),
    Single(VariableEl<'a>),
}

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq)]
pub struct Variable<'a> {
    inner: VariableInner<'a>,
}

impl<'a> Variable<'a> {
    #[must_use]
    fn from_segments(segments: Vec<VariableEl<'a>>) -> Self {
        Self {
            inner: VariableInner::Segments(segments),
        }
    }
    #[must_use]
    pub fn single_unchecked(name: impl Into<VariableEl<'a>>) -> Self {
        Self {
            inner: VariableInner::Single(name.into()),
        }
    }
    #[must_use]
    pub fn single(var: impl Into<VariableEl<'a>>) -> Self {
        let val = var.into();
        assert!(
            !val.contains('.'),
            "single cannot contain dot separator. Use parse if you want that"
        );
        Self::single_unchecked(val)
    }
    #[must_use]
    pub fn from_parts(parts: impl IntoIterator<Item = impl Into<VariableEl<'a>>>) -> Self {
        Self {
            inner: VariableInner::Segments(parts.into_iter().map(|p| p.into()).collect()),
        }
    }
}

impl FromStr for Variable<'static> {
    type Err = parse::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(parse::Error::new(
                (0, 0),
                parse::ErrorType::EmptyVariableSegment,
            ));
        }
        let chars = s.as_bytes();
        match parse::try_parse_variable_segment(chars) {
            Err(e) => Err(e),
            Ok(seg) => {
                let len = seg.len();
                let seg_s = unsafe { std::str::from_utf8_unchecked(seg) };
                Ok(if len == s.len() {
                    Self::single_unchecked(seg_s.to_owned())
                } else if chars[len] as char == ' ' {
                    return Err(parse::Error::new((len, 0), parse::ErrorType::SpaceInPath));
                } else {
                    let mut segments = vec![Cow::Owned(seg_s.to_owned())];
                    let mut head = seg_s.len();
                    Self::from_segments(loop {
                        if head == s.len() {
                            break segments;
                        }
                        assert!(head < s.len());
                        match parse::try_parse_variable_segment(&chars[head..]) {
                            Err(e) => return Err(e),
                            Ok(seg) => {
                                let len = seg.len();
                                segments.push(Cow::Owned(
                                    unsafe { std::str::from_utf8_unchecked(seg) }.to_owned(),
                                ));
                                head += len;
                            }
                        }
                    })
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_variable_from_str_errors_if_space_in_path() {
        let var = Variable::from_str("a .b");
        assert_eq!(
            var,
            Err(parse::Error::new((1, 0), parse::ErrorType::SpaceInPath))
        );
    }
    #[test]
    #[should_panic]
    fn constructing_single_variable_with_path_fails() {
        let _ = Variable::single("a.b");
    }
    #[test]
    fn parsing_variable_from_str_creates_single_if_only_one_element() {
        let var: Variable = "el".parse().unwrap();
        assert_eq!(var.inner, VariableInner::Single("el".into()));
    }
}
