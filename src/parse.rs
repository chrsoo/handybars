use crate::Variable;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub enum ErrorType {
    EmptyVariableSegment,
    NewlineInVariableSegment,
}
impl std::fmt::Display for ErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorType::EmptyVariableSegment => f.write_str("empty variable segment name"),
            ErrorType::NewlineInVariableSegment => f.write_str("newline in variable segment"),
        }
    }
}

#[derive(Debug)]
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

fn try_parse_variable_segment<'a>(input: &'a [u8]) -> Option<Result<&'a [u8]>> {
    for offset in 0..input.len() {
        let ch = input[offset];
        let pos = (offset, 0);
        match ch as char {
            '.' => {
                return Some(if offset == 0 {
                    Err(Error::new(pos, ErrorType::EmptyVariableSegment))
                } else {
                    Ok(&input[..offset])
                });
            }
            '\n' => return Some(Err(Error::new(pos, ErrorType::NewlineInVariableSegment))),
            _ => {}
        }
    }
    None
}

fn parse_template_inner<'a>(input: &'a [u8]) -> Option<Result<(Variable<'a>, usize)>> {
    let mut head = 0;
    let mut segments: Vec<&'a str> = Vec::new();
    let mut row = 0;
    let mut col = 0;
    while head < input.len() {
        let offset = (col as usize, row as usize);
        if input[head] as char == '}' && input[head + 1] as char == '}' {
            if segments.is_empty() {
                return Some(Err(Error::new(offset, ErrorType::EmptyVariableSegment)));
            }
            return Some(Ok((Variable::from_parts(segments), head + 2)));
        }
        match try_parse_variable_segment(&input[head..]) {
            Some(Ok(segment)) => segments.push(str_from_utf8(segment)),
            Some(Err(e)) => return Some(Err(e)),
            None => {}
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
            match parse_template_inner(&chars[head + 2..]) {
                Some(Ok((var, len))) => {
                    head += len + 2;
                    Some(var)
                }
                Some(Err(e)) => return Err(e.add_offset((pos.0 + 2, pos.1))),
                None => None,
            }
        } else {
            None
        };
        if let Some(var) = var {
            if tail != head {
                tokens.push(Token::Str(str_from_utf8(&chars[tail..head])))
            }
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
    if tail != head {
        tokens.push(Token::Str(str_from_utf8(&chars[tail..head])));
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
        assert_eq!(offset, s.len() - 2);
        assert_eq!(&var, &Variable::from_parts(["some", "txt"]));
    }
    #[test]
    fn parsing_template_extracts_engine_samples() {
        let parsed = tokenize("{{ var }}etc").unwrap();
        assert_eq!(
            parsed.as_slice(),
            &[
                Token::Variable(Variable::from_parts(vec!["var".to_owned()])),
                Token::Str("etc")
            ]
        );
    }
}
