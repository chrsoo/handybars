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
    fn from_segments(segments: Vec<VariableEl<'a>>) -> Self {
        Self {
            inner: VariableInner::Segments(segments),
        }
    }
    pub fn single_unchecked(name: impl Into<VariableEl<'a>>) -> Self {
        Self {
            inner: VariableInner::Single(name.into()),
        }
    }
    pub fn single(var: impl Into<VariableEl<'a>>) -> Self {
        let val = var.into();
        assert!(
            !val.contains('.'),
            "single cannot contain dot separator. Use parse if you want that"
        );
        Self::single_unchecked(val)
    }
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
            Some(Err(e)) => Err(e),
            Some(Ok(seg)) => {
                let seg_s = unsafe { std::str::from_utf8_unchecked(seg) };
                Ok(if seg.len() == s.len() {
                    Self::single_unchecked(seg_s.to_owned())
                } else {
                    let mut segments = vec![Cow::Owned(seg_s.to_owned())];
                    let mut head = seg_s.len();
                    Self::from_segments(loop {
                        if head == s.len() {
                            break segments;
                        }
                        assert!(head < s.len());
                        match parse::try_parse_variable_segment(&chars[head..]) {
                            Some(Err(e)) => return Err(e),
                            Some(Ok(seg)) => {
                                segments.push(Cow::Owned(
                                    unsafe { std::str::from_utf8_unchecked(seg) }.to_owned(),
                                ));
                                head += seg.len();
                            }
                            None => {
                                break segments;
                            }
                        }
                    })
                })
            }
            None => unreachable!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parsing_variable_from_str_creates_single_if_only_one_element() {
        let var: Variable = "el".parse().unwrap();
        assert_eq!(var.inner, VariableInner::Single("el".into()));
    }
}
