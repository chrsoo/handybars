use crate::Variable;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, PartialEq, Eq)]
pub enum ErrorType {
    EmptyVariableSegment,
    NewlineInVariableSegment,
    SpaceInPath,
}
impl std::fmt::Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorType::EmptyVariableSegment => f.write_str("empty variable segment name"),
            ErrorType::NewlineInVariableSegment => f.write_str("newline in variable segment"),
            ErrorType::SpaceInPath => f.write_str("space in variable path"),
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

pub(crate) fn try_parse_variable_segment(input: &[u8]) -> Result<&[u8]> {
    if input.is_empty() {
        return Err(Error::new((0, 0), ErrorType::EmptyVariableSegment));
    }
    let mut offset = 0;
    while offset < input.len() {
        let ch = input[offset];
        let pos = (offset, 0);
        match ch as char {
            '.' => {
                return if offset == 0 {
                    Err(Error::new(pos, ErrorType::EmptyVariableSegment))
                } else {
                    Ok(&input[..offset])
                };
            }
            '\n' => return Err(Error::new(pos, ErrorType::NewlineInVariableSegment)),
            _ if ch.is_ascii_punctuation()
                || ch.is_ascii_digit()
                || ch.is_ascii_control()
                || ch.is_ascii_graphic() =>
            {
                return Ok(&input[..offset])
            }
            _ => {}
        }
        offset += 1;
    }
    Ok(input)
}

fn parse_template_inner<'a>(input: &'a [u8]) -> Option<Result<(Variable<'a>, usize)>> {
    let mut head = 0;
    let mut segments: Vec<&'a str> = Vec::new();
    let mut row = 0;
    let mut col = 0;
    fn check_end_condition(head: usize, input: &[u8]) -> bool {
        input[head] as char == '}' && input[head + 1] as char == '}'
    }
    while head < input.len() - 1 {
        if input[head] as char != ' ' {
            let offset = (col as usize, row as usize);
            if check_end_condition(head, input) {
                if segments.is_empty() {
                    return Some(Err(Error::new(offset, ErrorType::EmptyVariableSegment)));
                }
                return Some(Ok((Variable::from_parts(segments), head + 2)));
            }
            if let Ok(segment) = try_parse_variable_segment(&input[head..]) {
                let segment = str_from_utf8(segment);
                let len = segment.len();
                assert!(
                    head + len < input.len() - 1,
                    "{head} + {len} >= {input_len} - 1. segment: '{segment}'",
                    input_len = input.len() - 1,
                );
                head += len;
                let old_head = head;
                while head < input.len() {
                    if input[head] as char == ' ' {
                        head += 1;
                    } else {
                        break;
                    }
                }
                if segments.is_empty() && check_end_condition(head, input) {
                    return Some(Ok((Variable::single_unchecked(segment), head + 2)));
                } else if input[len] as char == ' ' {
                    return Some(Err(Error::new((old_head, 0), ErrorType::SpaceInPath)));
                } else {
                    segments.push(segment);
                }
            }
        }
        head += 1;
        col += 1;
    }
    None
}
fn str_from_utf8(chars: &[u8]) -> &str {
    std::str::from_utf8(&chars).expect("This should never be hit, its a bug please investigate me")
}

pub fn tokenize(input: &str) -> Result<Vec<Token>> {
    if input.is_empty() {
        return Ok(Default::default());
    }
    let mut tokens = Vec::new();
    let mut head = 0;
    let mut tail = 0;
    let chars = input.as_bytes();
    let mut row = 0;
    let mut col = 0;
    while head < input.len() {
        let pos = (col, row);
        if head >= input.len() {
            break;
        }
        if head == input.len() - 1 {
            break;
        }
        let var = if chars[head] as char == '{' && chars[head + 1] as char == '{' {
            parse_template_inner(&chars[head + 2..])
                .transpose()
                .map_err(|e| e.add_offset((pos.0 + 2, pos.1)))?
        } else {
            None
        };
        if let Some((var, len)) = var {
            if tail != head {
                tokens.push(Token::Str(str_from_utf8(&chars[tail..head])))
            }
            head += len + 2;
            tail = head;
            tokens.push(Token::Variable(var));
        } else {
            if chars[head] as char == '\n' {
                col = 0;
                row += 1;
            } else {
                col += 1;
            }
            head += 1;
        }
    }
    if tail != input.len() - 1 {
        tokens.push(Token::Str(str_from_utf8(&chars[tail..])));
    }
    Ok(tokens)
}

#[derive(Debug, PartialEq, Eq)]
pub enum Token<'a> {
    Variable(Variable<'a>),
    Str(&'a str),
}

#[cfg(test)]
mod tests {
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
        let r = try_parse_variable_segment(input);
        assert_eq!(r, Ok("seg".as_bytes()))
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
}
