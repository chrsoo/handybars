#![warn(clippy::undocumented_unsafe_blocks)]
#![warn(clippy::unimplemented)]
#![warn(missing_docs)]

//!

use std::{borrow::Cow, str::FromStr};

mod context;
pub mod parse;
mod value;

pub use context::Context;
pub use value::{Object, Value};

use crate::parse::ErrorKind;

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

/// Variable that can be used in templates
///
/// A variable is a series of non-empty strings seperated by `.`
///
/// The lifetime specifier is used to allow variables
/// which do not own all of their parts. To get a variable
/// that _does_ own everything see [`into_owned`](Variable::into_owned)
///
#[repr(transparent)]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Variable<'a> {
    inner: VariableInner<'a>,
}

impl<'a> Variable<'a> {
    /// Convert a variable into one which owns all of its parts
    #[must_use]
    pub fn into_owned(self) -> Variable<'static> {
        Variable {
            inner: self.inner.into_owned(),
        }
    }
    /// Length of the variable in bytes, including seperators
    ///
    /// ```
    /// # use handybars::Variable;
    /// # use std::str::FromStr;
    /// let s = "a.b.c";
    /// let var = Variable::from_str(s).unwrap();
    /// assert_eq!(var.len(), s.len());
    /// ```
    #[must_use]
    #[allow(clippy::len_without_is_empty)] // impossible for variable to be empty
    pub fn len(&self) -> usize {
        match &self.inner {
            VariableInner::Segments(s) => s.iter().map(|s| s.len()).sum::<usize>() + (s.len() - 1),
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
    /// Construct a variable out of a single element
    ///
    /// Panics: If given a string which contains `.` or `var` is an empty string
    #[must_use]
    pub fn single(var: impl Into<VariableEl<'a>>) -> Self {
        let val = var.into();
        assert!(
            !val.contains('.'),
            "single cannot contain dot separator. Use parse if you want that"
        );
        assert!(
            !val.is_empty(),
            "cannot construct a variable with an empty string"
        );
        Self::single_unchecked(val)
    }
    /// Construct a variable from parts individually
    ///
    /// Panics: If any string in `parts` is empty
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
    /// Join together two variables
    ///
    /// ```
    /// # use handybars::Variable;
    /// let var = Variable::single("a").join(Variable::single("b"));
    /// assert_eq!(&var.to_string(), "a.b");
    /// ```
    #[must_use]
    pub fn join(self, other: Self) -> Self {
        match self.inner {
            VariableInner::Segments(mut xs) => match other.inner {
                VariableInner::Segments(mut ys) => {
                    xs.append(&mut ys);
                    Self::from_segments(xs)
                }
                VariableInner::Single(s) => {
                    xs.push(s);
                    Self::from_segments(xs)
                }
            },
            VariableInner::Single(s) => match other.inner {
                VariableInner::Segments(mut xs) => {
                    xs.insert(0, s);
                    Self::from_segments(xs)
                }
                VariableInner::Single(y) => Self::from_segments(vec![s, y]),
            },
        }
    }
}
impl<'a> std::fmt::Display for Variable<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.inner {
            VariableInner::Segments(s) => f.write_str(&s.join(".")),
            VariableInner::Single(s) => f.write_str(s),
        }
    }
}

fn parse_with_terminator<F: Fn(u8) -> bool>(
    s: &str,
    valid_pred: F,
    error_if_invalid: bool,
) -> Result<Variable<'static>, parse::Error> {
    let chars = s.as_bytes();

    let valid_len = {
        let mut head = 0;
        while head < chars.len() && valid_pred(chars[head]) {
            head += 1;
        }
        head
    };
    if error_if_invalid && valid_len != s.len() {
        return Err(parse::Error::new(
            (valid_len, 0),
            parse::ErrorKind::InvalidCharacter {
                token: chars[valid_len],
            },
        ));
    }
    if valid_len == 0 {
        return Err(parse::Error::new(
            (0, 0),
            parse::ErrorKind::EmptyVariableSegment,
        ));
    }

    match parse::try_parse_variable_segment(chars) {
        Err(e) => Err(e),
        Ok(seg) => {
            let len = seg.len();
            let seg_s = parse::str_from_utf8(seg);
            #[allow(clippy::blocks_in_if_conditions)]
            Ok(
                if {
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
                    (found_space || len == valid_len) && found_dot
                } {
                    return Err(parse::Error::new((len, 0), parse::ErrorKind::SpaceInPath));
                } else if len == valid_len {
                    Variable::single_unchecked(seg_s.to_owned())
                } else {
                    let mut segments = vec![Cow::Owned(seg_s.to_owned())];
                    let mut head = seg_s.len();
                    let mut segs = loop {
                        if head == valid_len || chars[head] as char == ' ' {
                            break segments;
                        }
                        if chars[head] as char == '.' {
                            let orig_head = head;
                            head += 1;
                            while head < chars.len() && chars[head] as char == ' ' {
                                head += 1;
                            }
                            if head == valid_len {
                                return Err(parse::Error::new(
                                    (orig_head, 0),
                                    ErrorKind::EmptyVariableSegment,
                                ));
                            }
                            continue;
                        }
                        assert!(head < s.len());
                        match parse::try_parse_variable_segment(&chars[head..]) {
                            Err(e) => return Err(e.add_offset((head, 0))),
                            Ok(seg) => {
                                let len = seg.len();
                                segments.push(Cow::Owned(parse::str_from_utf8(seg).to_owned()));
                                head += len;
                            }
                        }
                    };
                    if segs.len() == 1 {
                        Variable::single_unchecked(segs.pop().unwrap())
                    } else {
                        Variable::from_segments(segs)
                    }
                },
            )
        }
    }
}

impl FromStr for Variable<'static> {
    type Err = parse::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        parse_with_terminator(
            s,
            |ch| parse::is_valid_variable_name_ch(ch) || ch as char == ' ' || ch as char == '.',
            true,
        )
    }
}

#[cfg(test)]
mod tests {
    use proptest::{prop_assert_eq, proptest};

    use super::*;

    #[test]
    fn parsing_variable_from_str_errors_if_space_in_path() {
        let var = Variable::from_str("a .b");
        assert_eq!(
            var,
            Err(parse::Error::new((1, 0), parse::ErrorKind::SpaceInPath))
        );
    }
    #[test]
    fn variable_join_reuses_vec_on_either_side() {
        let v1 = Variable::single("v");
        let v2 = Variable::from_parts(["t", "t2"]);
        let v3 = v1.clone().join(v2.clone());
        assert_eq!(
            v3.inner,
            VariableInner::Segments(vec!["v".into(), "t".into(), "t2".into()])
        );
        let v3 = v2.join(v1);
        assert_eq!(
            v3.inner,
            VariableInner::Segments(vec!["t".into(), "t2".into(), "v".into()])
        );
    }
    #[test]
    #[should_panic]
    fn constructing_single_variable_with_path_fails() {
        let _ = Variable::single("a.b");
    }

    #[test]
    #[should_panic]
    fn constructing_single_variable_with_empty_fails() {
        let _ = Variable::single("");
    }
    #[test]
    fn parsing_variable_from_str_creates_single_if_only_one_element() {
        let var: Variable = "el".parse().unwrap();
        assert_eq!(var.inner, VariableInner::Single("el".into()));
    }

    #[test]
    fn parsing_variable_with_trailing_dot_fails() {
        assert_eq!(
            Variable::from_str("x."),
            Err(parse::Error::new(
                (1, 0),
                parse::ErrorKind::EmptyVariableSegment
            ))
        );
    }

    #[test]
    fn parsing_variable_from_path_works() {
        let var: Variable = "x.y".parse().unwrap();
        assert_eq!(var, Variable::from_parts(["x", "y"]));
    }
    fn run_parsing_variable_test(input: &str) -> (Result<Variable, parse::Error>, Variable) {
        let var = Variable::from_str(input);
        let split = input
            .split(' ')
            .next()
            .unwrap()
            .trim_end_matches('}')
            .split('.')
            .collect::<Vec<_>>();
        let expected = if split.len() == 1 {
            Variable::single_unchecked(split[0])
        } else {
            Variable::from_parts(split)
        };
        (var, expected)
    }

    proptest! {
        #[test]
        fn parsing_variable_from_ascii_works(input in r"([[:alpha:]]\d)+(\.([[[:alpha:]]\d])+)*[ ]*") {
            let (var, expected) = run_parsing_variable_test(&input);
            prop_assert_eq!(var, Ok(expected));
        }
        #[test]
        fn parsing_variable_from_unicode_works(input in r"([[[:alpha:]]~~[\p{Alphabetic}\d]])+(\.([[[:alpha:]]~~[\p{Alphabetic}\d]])+)*[ ]*") {
            let (var, expected) = run_parsing_variable_test(&input);
            prop_assert_eq!(var, Ok(expected));
        }
    }
}
