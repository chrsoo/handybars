

use crate::Variable;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorType {
    EmptyVariableSegment,
    NewlineInVariableSegment,
    SpaceInPath,
    InvalidCharacter { token: u8 },
}
impl std::fmt::Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorType::EmptyVariableSegment => f.write_str("empty variable segment name"),
            ErrorType::NewlineInVariableSegment => f.write_str("newline in variable segment"),
            ErrorType::SpaceInPath => f.write_str("space in variable path"),
            ErrorType::InvalidCharacter { token } => {
                f.write_fmt(format_args!("invalid character: '{token}'"))
            }
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Error {
    pub offset: (usize, usize),
    pub ty: ErrorType,
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (col, line) = self.offset;
        write!(
            f,
            "{} at line {line} column {col}",
            self.ty,
            line = line + 1,
            col = col + 1
        )
    }
}

impl Error {
    pub fn new(offset: (usize, usize), ty: ErrorType) -> Self {
        Self { offset, ty }
    }
    pub fn add_offset(mut self, offset: (usize, usize)) -> Self {
        self.offset.0 += offset.0;
        self.offset.1 += offset.1;
        self
    }
}
pub(crate) fn is_valid_variable_name_ch(ch: u8) -> bool {
    !(ch.is_ascii_punctuation() || ch.is_ascii_control() || ch.is_ascii_whitespace())
}

pub(crate) fn try_parse_variable_segment(input: &[u8]) -> Result<&[u8]> {
    if input.is_empty() {
        return Err(Error::new((0, 0), ErrorType::EmptyVariableSegment));
    }
    let mut offset = 0;
    while offset < input.len() {
        let ch = input[offset];
        let pos = (offset, 0);
        match ch as char {
            '\n' => return Err(Error::new(pos, ErrorType::NewlineInVariableSegment)),
            _ if !is_valid_variable_name_ch(ch) => {
                return if offset == 0 {
                    Err(Error::new(pos, ErrorType::EmptyVariableSegment))
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

fn parse_template_inner<'a>(input: &'a [u8]) -> Option<Result<(Variable<'a>, usize)>> {
    let mut head = 0;
    while head < input.len() && input[head] as char == ' ' {
        head += 1;
    }
    let var = match super::parse_with_terminator(
        str_from_utf8(&input[head..]),
        |ch| ch as char != '}',
        false,
    ) {
        Ok(v) => v,
        Err(Error {
            ty: ErrorType::EmptyVariableSegment,
            offset: (0, 0),
        }) => return None,
        Err(e) => return Some(Err(e.add_offset((head, 0)))),
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
fn str_from_utf8(chars: &[u8]) -> &str {
    std::str::from_utf8(chars).expect("This should never be hit, its a bug please investigate me")
}

pub struct TokenizeIter<'a> {
    chars: &'a [u8],
    head: usize,
    tail: usize,
    row: usize,
    col: usize,
    hit_error: bool,
    var_next: Option<Variable<'a>>,
}

impl<'a> TokenizeIter<'a> {
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

impl<'a> Iterator for TokenizeIter<'a> {
    type Item = Result<Token<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
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
        if let Some(next) = self.var_next.take() {
            return Some(Ok(Token::Variable(next)));
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

pub fn tokenize(input: &str) -> Result<Vec<Token>> {
    TokenizeIter::new(input).collect()
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token<'a> {
    Variable(Variable<'a>),
    Str(&'a str),
}

#[cfg(test)]
mod tests {
    use proptest::{prop_assert_eq, proptest};

    use super::*;

    #[test]
    fn parse_template_inner_errors_with_space_in_path() {
        let r = parse_template_inner("x .y}}".as_bytes()).unwrap();
        assert_eq!(
            r,
            Err(Error {
                offset: (1, 0),
                ty: ErrorType::SpaceInPath
            })
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
}
