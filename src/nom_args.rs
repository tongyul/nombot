#[derive(Debug, PartialEq, Eq)]
pub enum Arg {
    Pos(String),
    Kw(String, String),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Command {
    pub name: String,
    pub args: Vec<Arg>,
    pub rest: String,
}

impl Command {
    pub fn new(name: String, args: Vec<Arg>, rest: String) -> Self { Self { name, args, rest } }
}

#[derive(Debug, PartialEq, Eq)]
pub struct ParseError {
    loc: usize,
    why: String,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "At position {} after prefix: {}", self.loc, self.why)
    }
}

impl ParseError {
    pub fn new(loc: usize, why: String) -> Self { Self { loc, why } }
    pub fn max_by_loc(self, other: Self) -> Self {
        if self.loc >= other.loc { self } else { other }
    }
}

pub fn parse(s: &str) -> Result<Command, ParseError> {
    let (name, mut s, mut offset) = expect_ident(s, 0)?;
    let mut args = vec![];
    let mut most_successful_error: Option<ParseError> = None;
    loop {
        if let Ok(s) = expect_rest(s, offset)
            .or_else(|_| expect_spaces(s, offset).and_then(|(s, o)| expect_rest(s, o))) {
            break Ok(Command::new(name, args, s))
        }
        let (s1, offset1) = match expect_spaces(s, offset) {
            Ok(x) => x,
            Err(e) =>
                break Err(match most_successful_error {
                    Some(m) => m.max_by_loc(e),
                    None => e,
                }),
        };
        match expect_key_value(s1, offset1) {
            Ok((k, v, r, o)) => {
                args.push(Arg::Kw(k, v));
                (s, offset) = (r, o);
            }
            Err(e1) => match expect_value(s1, offset1) {
                Ok((v, r, o)) => {
                    most_successful_error =
                        Some(match most_successful_error {
                            Some(m) => m.max_by_loc(e1),
                            None => e1,
                        });
                    args.push(Arg::Pos(v));
                    (s, offset) = (r, o);
                }
                Err(e2) =>
                    break Err(match most_successful_error {
                        Some(m) => m.max_by_loc(e1).max_by_loc(e2),
                        None => e1.max_by_loc(e2),
                    }),
            }
        }
    }
}

fn expect_rest(s: &str, offset: usize) -> Result<String, ParseError> {
    if s.chars().count() == 0 {
        Ok("".into())
    } else if s.chars().nth(0).unwrap() == '/' {
        Ok((&s[1..]).into())
    } else {
        Err(ParseError::new(offset, "expected a <rest> region beginning with '/'".into()))
    }
}

fn expect_key_value(s: &str, offset: usize) -> Result<(String, String, &str, usize), ParseError> {
    // eprintln!("expect_key_value({:?}, {:?})", s, offset);
    expect_ident(s, offset).and_then(
        |(k, s, offset)|
            if s.chars().count() >= 1 && s.chars().nth(0).unwrap() == '=' {
                expect_value(&s[1..], offset + 1).map(|(v, s, offset)| (k, v, s, offset))
            } else {
                Err(ParseError::new(offset, "expected '=' for key-value pair".into()))
            }
    )
}

fn expect_value(mut s: &str, mut offset: usize) -> Result<(String, &str, usize), ParseError> {
    let mut buf = vec![];

    loop {
        match expect_string(s, offset) {
            Ok((t, u, p)) => {
                buf.push(t);
                s = u;
                offset = p;
            }
            Err(e) => break if buf.len() == 0 {
                Err(e)
            } else {
                Ok((buf.join(""), s, offset))
            }
        }
    }
}

fn expect_string(s: &str, offset: usize) -> Result<(String, &str, usize), ParseError> {
    if s.chars().count() == 0 {
        return Err(ParseError::new(offset, "missing string".into()));
    }
    let first = s.chars().nth(0).unwrap();

    if first == '\'' || first == '"' {
        expect_quoted_string(s, offset)
    } else {
        expect_naked_string(s, offset)
    }
}

fn expect_quoted_string(s: &str, offset: usize) -> Result<(String, &str, usize), ParseError> {
    let quo: char;
    let mut end = 0usize;
    let mut buf = String::new();
    let mut itr = s.chars();
    if let Some(c) = itr.next() {
        if c == '\'' || c == '"' {
            quo = c;
            end += c.len_utf8();
        } else {
            return Err(ParseError::new(offset + end, "quoted string does not begin with quote".into()));
        }
    } else {
        return Err(ParseError::new(offset + end, "missing string".into()));
    }
    while let Some(c) = itr.next() {
        end += c.len_utf8();
        if c == quo {
            break;
        } else if c != '\\' {
            buf.push(c);
        } else /* c == '\\' */ {
            if let Some(d) = itr.next() {
                end += d.len_utf8();
                if d == '\'' || d == '"' || d == '\\' {
                    buf.push(d);
                } else if d == 'n' {
                    buf.push('\n');
                } else if d == 't' {
                    buf.push('\t');
                } else if d.is_whitespace() || d == '=' || d == '/' {
                    return Err(ParseError::new(offset + end, "\\(whitespace), \\=, and \\/ are only available in naked strings".into()));
                } else {
                    return Err(ParseError::new(offset + end, format!("unsupported escape sequence \\{}", d)));
                }
            } else {
                return Err(ParseError::new(offset + end, "unterminated quoted string".into()));
            }
        }
    }

    Ok((buf, &s[end..], offset + end))
}

fn expect_naked_string(s: &str, offset: usize) -> Result<(String, &str, usize), ParseError> {
    let mut end = 0usize;
    let mut buf = String::new();
    let mut itr = s.chars();
    while let Some(c) = itr.next() {
        if !c.is_whitespace() && c != '=' && c != '\'' && c != '"' && c != '/' && c != '\\' {
            end += c.len_utf8();
            buf.push(c);
        } else if c == '\\' {
            end += c.len_utf8();
            if let Some(d) = itr.next() {
                end += d.len_utf8();
                if d.is_whitespace() || d == '=' || d == '\'' || d == '"' || d == '/' || d == '\\' {
                    buf.push(d);
                } else if d == 'n' || d == 't' {
                    return Err(ParseError::new(offset + end, "\\n and \\t are only available in quoted strings".into()));
                } else {
                    return Err(ParseError::new(offset + end, format!("unsupported escape sequence \\{}", d)));
                }
            } else {
                return Err(ParseError::new(offset + end, "missing operand after escape character '\\'".into()));
            }
        } else {
            if end == 0 {
                return Err(ParseError::new(offset + end, format!("invalid character to appear in a naked string ({:?})", c)));
            } else {
                break;
            }
        }
    }

    Ok((buf, &s[end..], offset + end))
}

fn expect_ident(s: &str, offset: usize) -> Result<(String, &str, usize), ParseError> {
    let end = s.chars()
        .take_while(
            |&c| ('0' <= c && c <= '9')
                || ('A' <= c && c <= 'Z')
                || ('a' <= c && c <= 'z')
                || c == '_'
                || c == '-'
        )
        .map(char::len_utf8).sum();

    if end == 0 {
        Err(ParseError::new(offset, "expected an identifier, which begins with [0-9A-Za-z_-]".into()))
    } else {
        Ok(((&s[..end]).into(), &s[end..], offset + end))
    }
}

fn expect_spaces(s: &str, offset: usize) -> Result<(&str, usize), ParseError> {
    let end = s.chars().take_while(|&c| c.is_whitespace()).map(char::len_utf8).sum();

    if end == 0 {
        Err(ParseError::new(offset, "expected whitespace".into()))
    } else {
        Ok((&s[end..], offset + end))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_parse() {
        assert_eq!(
            parse("nom"),
            Ok(Command::new("nom".into(), vec![], "".into())),
        );
        assert_eq!(
            parse("clac -e/7 7 / 11 12 + +"),
            Ok(Command::new("clac".into(), vec![Arg::Pos("-e".into())], "7 7 / 11 12 + +".into())),
        );
        assert_eq!(
            parse("echo hello, world!"),
            Ok(Command::new(
                "echo".into(),
                vec![Arg::Pos("hello,".into()), Arg::Pos("world!".into())],
                "".into()
            )),
        );
        assert_eq!(
            parse("uwuify kaomoji-when=never/Euthanize me, sensei!"),
            Ok(Command::new(
                "uwuify".into(),
                vec![Arg::Kw("kaomoji-when".into(), "never".into())],
                "Euthanize me, sensei!".into()
            )),
        );
    }
}
