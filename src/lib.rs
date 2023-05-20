use std::{borrow::Cow, str::FromStr};

mod context;
mod parse;
mod value;

type VariableEl<'a> = Cow<'a, str>;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
enum VariableInner<'a> {
    Segments(Vec<VariableEl<'a>>),
    Single(VariableEl<'a>),
}
impl<'a> VariableInner<'a> {
    fn into_owned(self) -> VariableInner<'static> {
        match self {
            VariableInner::Segments(s) => {
                VariableInner::Segments(s.into_iter().map(|s| Cow::Owned(s.into_owned())).collect())
            }
            VariableInner::Single(s) => VariableInner::Single(Cow::Owned(s.into_owned())),
        }
    }
}

#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Variable<'a> {
    inner: VariableInner<'a>,
}

impl<'a> Variable<'a> {
    #[must_use]
    pub fn into_owned(self) -> Variable<'static> {
        Variable {
            inner: self.inner.into_owned(),
        }
    }
    #[must_use]
    #[allow(clippy::len_without_is_empty)] // impossible for variable to be empty
    pub fn len(&self) -> usize {
        match &self.inner {
            VariableInner::Segments(s) => s.iter().map(|s| s.len()).sum(),
            VariableInner::Single(s) => s.len(),
        }
    }
    #[must_use]
    fn from_segments(segments: Vec<VariableEl<'a>>) -> Self {
        Self {
            inner: VariableInner::Segments(segments),
        }
    }
    #[must_use]
    fn single_unchecked(name: impl Into<VariableEl<'a>>) -> Self {
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
            inner: VariableInner::Segments(
                parts
                    .into_iter()
                    .map(|p| p.into())
                    .inspect(|s| assert!(!s.is_empty(), "variable part cannot be empty"))
                    .collect(),
            ),
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
                #[allow(clippy::blocks_in_if_conditions)]
                Ok(if len == s.len() {
                    Self::single_unchecked(seg_s.to_owned())
                } else if {
                    let mut found_space = false;
                    let mut found_dot = false;
                    for c in &chars[len..] {
                        match *c as char {
                            ' ' => found_space = true,
                            '.' => {
                                found_dot = true;
                                break;
                            }
                            _ => break,
                        }
                    }
                    found_space && found_dot
                } {
                    return Err(parse::Error::new((len, 0), parse::ErrorType::SpaceInPath));
                } else {
                    let mut segments = vec![Cow::Owned(seg_s.to_owned())];
                    let mut head = seg_s.len();
                    let mut segs = loop {
                        if head == s.len() || chars[head] as char == ' ' {
                            break segments;
                        }
                        if chars[head] as char == '.' {
                            head += 1;
                            continue;
                        }
                        assert!(head < s.len());
                        match parse::try_parse_variable_segment(&chars[head..]) {
                            Err(e) => return Err(e.add_offset((head, 0))),
                            Ok(seg) => {
                                let len = seg.len();
                                segments.push(Cow::Owned(
                                    unsafe { std::str::from_utf8_unchecked(seg) }.to_owned(),
                                ));
                                head += len;
                            }
                        }
                    };
                    if segs.len() == 1 {
                        Self::single_unchecked(segs.pop().unwrap())
                    } else {
                        Self::from_segments(segs)
                    }
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use proptest::{prelude::Arbitrary, prop_assert, prop_assert_eq, proptest, strategy::Strategy};

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

    #[test]
    fn parsing_variable_from_path_works() {
        let var: Variable = "x.y".parse().unwrap();
        assert_eq!(var, Variable::from_parts(["x", "y"]));
    }
    proptest! {
        #[test]
        fn parsing_variable_from_unicode_works(input in r"([[[:alpha:]]~~[\p{Alphabetic}\d]])+(\.([[[:alpha:]]~~[\p{Alphabetic}\d]])+)*") {
            let var = Variable::from_str(&input);
            let split = input.split('.').collect::<Vec<_>>();
            let expected = if split.len() == 1 {
                Variable::single_unchecked(split[0])
            }else {
                Variable::from_parts(split)
            };
            prop_assert_eq!(var, Ok(expected));
        }
    }
}
