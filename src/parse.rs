//! Parsing utilities for templates
use crate::Variable;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Copy, Default)]
/// Location information
///
/// Used for offsets into source. All indexes are 0 based (i.e. row 0, col 0 is the first character of the first line)
pub struct Location {
    #[allow(missing_docs)] // seriousely, I don't think this one needs explaining
    pub line: usize,
    #[allow(missing_docs)]
    pub col: usize,
}
impl Location {
    /// Construct a new Location
    pub fn new(col: usize, line: usize) -> Self {
        Self { col, line }
    }
    /// (0, 0)
    pub fn zero() -> Self {
        Self::default()
    }
}
impl From<(usize, usize)> for Location {
    fn from(value: (usize, usize)) -> Self {
        Self::new(value.0, value.1)
    }
}

impl std::ops::Add for Location {
    type Output = Location;

    fn add(self, rhs: Self) -> Self::Output {
        Self {
            line: self.line + rhs.line,
            col: rhs.col + self.col,
        }
    }
}
impl std::ops::AddAssign for Location {
    fn add_assign(&mut self, rhs: Self) {
        self.col += rhs.col;
        self.line += rhs.line;
    }
}
impl std::ops::SubAssign for Location {
    fn sub_assign(&mut self, rhs: Self) {
        self.col -= rhs.col;
        self.line -= rhs.line;
    }
}
impl std::ops::Sub for Location {
    type Output = Location;

    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            line: self.line - rhs.line,
            col: self.col - rhs.col,
        }
    }
}
impl std::fmt::Display for Location {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!(
            "(row: {r}, column: {c})",
            r = self.line,
            c = self.col
        ))
    }
}

/// Kind of error reported by parsers
#[derive(Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ErrorKind {
    /// Variable segment is empty
    EmptyVariableSegment,
    /// Newline within a path variable
    NewlineInVariableSegment,
    /// Space inside path variable (variable with at least one `.`)
    SpaceInPath,
    /// Invalid character encountered
    InvalidCharacter {
        #[allow(missing_docs)]
        token: u8,
    },
    /// More than 1 variable in a template (`{{ ... }}`) block
    TooManyVariablesInBlock,
}
impl std::fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorKind::EmptyVariableSegment => f.write_str("empty variable segment name"),
            ErrorKind::NewlineInVariableSegment => f.write_str("newline in variable segment"),
            ErrorKind::SpaceInPath => f.write_str("space in variable path"),
            ErrorKind::InvalidCharacter { token } => {
                f.write_fmt(format_args!("invalid character: '{token}'"))
            }
            ErrorKind::TooManyVariablesInBlock => {
                f.write_str("more than 1 variable in template block")
            }
        }
    }
}
impl std::error::Error for Error {}

/// Type for errors reported by parsing
#[derive(Debug, PartialEq, Eq)]
pub struct Error {
    /// Offset into source for error
    ///
    /// First is column, second is row
    offset: Location,
    /// Type of error
    ty: ErrorKind,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} at line {line} column {col}",
            self.ty,
            line = self.offset.line + 1,
            col = self.offset.col + 1
        )
    }
}

impl Error {
    /// Construct a new error with offset and kind
    pub fn new(offset: impl Into<Location>, ty: ErrorKind) -> Self {
        Self {
            offset: offset.into(),
            ty,
        }
    }
    /// Add offset to existing error
    pub fn add_offset(mut self, offset: impl Into<Location>) -> Self {
        self.offset += offset.into();
        self
    }

    /// What kind of error is this
    pub fn kind(&self) -> &ErrorKind {
        &self.ty
    }

    /// Location in the input that this error ocurred
    pub fn location(&self) -> Location {
        self.offset
    }
}
pub(crate) fn is_valid_identifier_ch(ch: u8) -> bool {
    !(ch.is_ascii_whitespace()
        || matches!(
            ch as char,
            '!' | '"'
                | '#'
                | '%'
                | '&'
                | '\''
                | '('
                | ')'
                | '*'
                | '+'
                | '|'
                | '.'
                | '/'
                | ';'
                | '<'
                | '='
                | '>'
                | '@'
                | '['
                | ']'
                | '\\'
                | '^'
                | '`'
                | '{'
                | '}'
                | ','
                | '~'
        ))
}

pub(crate) fn try_parse_variable_segment(input: &[u8]) -> Result<&[u8]> {
    if input.is_empty() {
        return Err(Error::new(
            Location::zero(),
            ErrorKind::EmptyVariableSegment,
        ));
    }
    let mut offset = 0;
    while offset < input.len() {
        let ch = input[offset];
        let pos = Location {
            line: 0,
            col: offset,
        };
        match ch as char {
            '\n' => return Err(Error::new(pos, ErrorKind::NewlineInVariableSegment)),
            _ if !is_valid_identifier_ch(ch) => {
                return if offset == 0 {
                    Err(Error::new(pos, ErrorKind::EmptyVariableSegment))
                } else {
                    Ok(&input[..offset])
                };
            }
            _ => {}
        }
        offset += 1;
    }
    Ok(input)
}

fn parse_template_inner(input: &[u8]) -> Option<Result<(Variable, usize)>> {
    let mut head = 0;
    while head < input.len() && input[head] as char == ' ' {
        head += 1;
    }
    let var = match super::parse_with_terminator(str_from_utf8(&input[head..]), false) {
        Ok(v) => v,
        Err(Error {
            ty: ErrorKind::EmptyVariableSegment,
            offset: Location { line: 0, col: 0 },
        }) => return None,
        Err(e) => return Some(Err(e.add_offset(Location::new(head, 0)))),
    };
    head += var.len();
    fn check_end_condition(head: usize, input: &[u8]) -> bool {
        input[head] as char == '}' && input[head + 1] as char == '}'
    }
    while head < input.len() - 1 {
        if check_end_condition(head, input) {
            return Some(Ok((var, head + 2)));
        }
        head += 1;
    }
    None
}

#[inline]
pub(crate) fn str_from_utf8(chars: &[u8]) -> &str {
    #[cfg(debug_assertions)]
    {
        std::str::from_utf8(chars).expect(
            "failed to convert input to utf8, this is a bug fixme or it'll be UB in release mode",
        )
    }
    #[cfg(not(debug_assertions))]
    {
        // Safety: This is ok because we only ever call it on slices of strings, separated by ascii characters
        unsafe { std::str::from_utf8_unchecked(chars) }
    }
}

/// Tokenization iterator
///
/// This exists to allow true zero-allocation tokenization. See [`tokenize`](crate::parse::tokenize) for
/// a version of this which gives you a vector and result.
///
/// ```
/// # use handybars::{*, parse::*};
/// let mut tokens = Tokenize::new("some {{ text }}");
/// assert_eq!(tokens.next(), Some(Ok(Token::Str("some "))));
/// assert_eq!(tokens.next(), Some(Ok(Token::Variable(Variable::single("text")))));
/// assert_eq!(tokens.next(), None);
/// ```
///
/// Note: Once this returns `Some(Err(_))` once it will always return `None` after
///
/// ```
/// # use handybars::{*, parse::*};
/// let mut tokens = Tokenize::new("{{ i.am.invalid. }} I would appear if not for the error");
/// assert!(matches!(tokens.next(), Some(Err(_))));
/// assert_eq!(tokens.next(), None);
/// ```
///
pub struct Tokenize<'a> {
    chars: &'a [u8],
    head: usize,
    tail: usize,
    row: usize,
    col: usize,
    hit_error: bool,
    var_next: Option<Variable<'a>>,
}

impl<'a> Tokenize<'a> {
    /// Construct a new Tokenize iterator on a given input
    pub fn new(input: &'a str) -> Self {
        Self {
            chars: input.as_bytes(),
            head: 0,
            tail: 0,
            row: 0,
            col: 0,
            hit_error: false,
            var_next: None,
        }
    }
}

impl<'a> Iterator for Tokenize<'a> {
    type Item = Result<Token<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.var_next.take() {
            return Some(Ok(Token::Variable(next)));
        }
        if self.hit_error {
            return None;
        } else if self.head >= self.chars.len() {
            if self.tail < self.chars.len() - 1 {
                let val = Some(Ok(Token::Str(str_from_utf8(&self.chars[self.tail..]))));
                self.tail = self.chars.len() - 1;
                return val;
            }
            return None;
        }

        while self.head < self.chars.len() {
            let pos = (self.col, self.row);
            let var = if self.chars[self.head] as char == '{'
                && self.chars[self.head + 1] as char == '{'
            {
                parse_template_inner(&self.chars[self.head + 2..])
                    .transpose()
                    .map_err(|e| e.add_offset((pos.0 + 2, pos.1)))
                    .transpose()
            } else {
                None
            };
            match var {
                Some(Ok((var, len))) => {
                    let prev_tail = self.tail;
                    let prev_head = self.head;
                    let should_add_prev = self.tail != self.head;
                    self.head += len + 2;
                    self.tail = self.head;
                    self.col += len + 2;
                    if should_add_prev {
                        self.var_next.replace(var);
                        let val = Some(Ok(Token::Str(str_from_utf8(
                            &self.chars[prev_tail..prev_head],
                        ))));
                        return val;
                    } else {
                        return Some(Ok(Token::Variable(var)));
                    }
                }
                Some(Err(e)) => {
                    self.hit_error = true;
                    return Some(Err(e));
                }
                None => {
                    if self.chars[self.head] as char == '\n' {
                        self.col = 0;
                        self.row += 1;
                    } else {
                        self.col += 1;
                    }
                    self.head += 1;
                }
            }
        }
        if self.tail != self.chars.len() - 1 {
            let val = Some(Ok(Token::Str(str_from_utf8(&self.chars[self.tail..]))));
            self.tail = self.chars.len() - 1;
            return val;
        }
        None
    }
}

/// Tokenize an input with allocation
pub fn tokenize(input: &str) -> Result<Vec<Token>> {
    Tokenize::new(input).collect()
}

/// Type for tokens emitted by the parser
#[derive(Debug, PartialEq, Eq)]
pub enum Token<'a> {
    /// Variable for later expansion
    Variable(Variable<'a>),
    /// Untemplated string input
    Str(&'a str),
}

#[cfg(test)]
mod tests {
    use proptest::{prop_assert_eq, proptest};

    use super::*;

    #[test]
    fn parse_template_inner_errors_with_space_in_path() {
        let r = parse_template_inner("x .y}}".as_bytes()).unwrap();
        assert_eq!(r, Err(Error::new((1, 0), ErrorKind::SpaceInPath)));
    }

    #[test]
    fn parsing_tokens_with_space_before_template_works() {
        let tokens = tokenize("some {{ text }}").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Str("some "),
                Token::Variable(Variable::single("text"))
            ]
        );
    }

    #[test]
    fn invalid_template_causes_tokenize_to_halt() {
        let tokens = tokenize("{{invalid. }} some text");
        assert_eq!(
            tokens,
            Err(Error::new((9, 0), ErrorKind::EmptyVariableSegment))
        );
    }

    #[test]
    fn parsing_template_works_without_spaces() {
        let tokens = tokenize("{{test}}");
        assert_eq!(tokens, Ok(vec![Token::Variable(Variable::single("test"))]));
    }

    #[test]
    fn parse_segment_stops_on_non_alphanumeric_chars() {
        let r = try_parse_variable_segment("x}".as_bytes()).map(str_from_utf8);
        assert_eq!(r, Ok("x"));
    }
    #[test]
    fn parse_segment_strips_trailing_spaces_in_singleton_case() {
        let r = try_parse_variable_segment("x ".as_bytes()).map(str_from_utf8);
        assert_eq!(r, Ok("x"));
    }
    #[test]
    fn parse_segment_parses_no_separator_case() {
        let input = "seg".as_bytes();
        let r = try_parse_variable_segment(input);
        assert_eq!(r, Ok(input))
    }

    #[test]
    fn parse_segment_parses_with_seperator_returns_up_to_seperator() {
        let input = "seg.part.2".as_bytes();
        let r = try_parse_variable_segment(input).map(str_from_utf8);
        assert_eq!(r, Ok("seg"))
    }
    #[test]
    fn parse_with_equals_works() {
        let s = r"SOME_VAR={{ t1 }}
export THING=$SOME_VAR";
        let tkns = tokenize(s).unwrap();
        assert_eq!(
            tkns.as_slice(),
            &[
                Token::Str("SOME_VAR="),
                Token::Variable(Variable::single("t1".to_string())),
                Token::Str(
                    r"
export THING=$SOME_VAR"
                )
            ]
        )
    }

    #[test]
    fn parse_template_inner_parses_the_start_of_a_template() {
        let s = "some.txt }}h1";
        let cs = s.as_bytes();
        let (var, offset) = parse_template_inner(cs).unwrap().unwrap();
        assert_eq!(offset, s.len() - 2, "stops at template end");
        assert_eq!(
            &var,
            &Variable::from_parts(["some", "txt"]),
            "strips spaces"
        );
    }
    #[test]
    fn parsing_template_extracts_engine_samples() {
        let parsed = tokenize("{{ var }}etc").unwrap();
        assert_eq!(
            parsed.as_slice(),
            &[Token::Variable(Variable::single("var")), Token::Str("etc")]
        );
    }
    proptest! {
        #[test]
        fn parse_template_inner_allows_any_amount_of_whitespace(whitespace in "[ ]*") {
            let s = "test".to_owned() + &whitespace + "}}";
            let cs = s.as_bytes();
            let (var, _) = parse_template_inner(cs).unwrap().unwrap();
            prop_assert_eq!(
                &var,
                &Variable::single("test")
            );
        }
    }

    #[test]
    fn location_adds_correctly() {
        assert_eq!(
            Location { line: 2, col: 0 } + Location { line: 1, col: 3 },
            Location { line: 3, col: 3 }
        );
    }

    #[test]
    fn location_subtracts_correctly() {
        assert_eq!(
            Location { line: 5, col: 3 } - Location { line: 1, col: 2 },
            Location { line: 4, col: 1 }
        );
    }
}
