use std::{
    io::{self, Read},
    string::FromUtf8Error,
    vec::IntoIter,
};

use crate::string_reader::StringReader;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expression {
    Tuple(Vec<Expression>),
    Literal(String),
    String(String),
    BackQuotedString(String),
    Integer(u32),
}

pub type RandIter = IntoIter<Expression>;

#[derive(Debug)]
pub enum ApplicationError {
    MissingOperator,
    HeadIsNotLiteral,
}

pub type ApplicationResult<T> = Result<T, ApplicationError>;

pub fn expect_application(tuple: Vec<Expression>) -> ApplicationResult<(String, RandIter)> {
    let mut iter = tuple.into_iter();
    if let Some(rator) = iter.next() {
        match rator {
            Expression::Literal(l) => Ok((l, iter)),
            _ => Err(ApplicationError::HeadIsNotLiteral),
        }
    } else {
        Err(ApplicationError::MissingOperator)
    }
}

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    Utf8Error(FromUtf8Error),
    ParseError(ParseError),
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedEOF,
    UnexpectedCharacter(u8),
}

pub type ParseResult<T> = Result<T, Error>;

pub struct SExpParser<R: Read> {
    reader: StringReader<R>,
}

enum ExpressionOrChr {
    Expression(Expression),
    Chr(u8),
}

impl<R: Read> SExpParser<R> {
    pub const fn new(reader: StringReader<R>) -> Self {
        Self { reader }
    }

    const fn chr(&self) -> Option<u8> {
        self.reader.chr()
    }

    fn seek(&mut self) -> ParseResult<()> {
        self.reader.seek().map_err(Error::IOError)
    }

    pub fn parse_expression(&mut self) -> ParseResult<Expression> {
        self.parse_expression_or_chr().and_then(|eoc| match eoc {
            ExpressionOrChr::Expression(e) => Ok(e),
            ExpressionOrChr::Chr(c) => unexpected_chr(c),
        })
    }

    fn parse_expression_or_chr(&mut self) -> ParseResult<ExpressionOrChr> {
        let chr = self.roll_up_and_get()?;
        self.seek()?;
        match chr {
            b'(' => self.parse_tuple().map(ExpressionOrChr::Expression),
            b'"' => self.parse_string().map(ExpressionOrChr::Expression),
            b'`' => self
                .parse_backquoted_string()
                .map(ExpressionOrChr::Expression),
            b'0'..=b'9' => self.parse_number(chr).map(ExpressionOrChr::Expression),
            b'a'..=b'z' | b'A'..=b'Z' => self.parse_literal(chr).map(ExpressionOrChr::Expression),
            _ => Ok(ExpressionOrChr::Chr(chr)),
        }
    }

    fn parse_tuple(&mut self) -> ParseResult<Expression> {
        let mut result = Vec::new();

        loop {
            let node = self.parse_expression_or_chr()?;
            match node {
                ExpressionOrChr::Expression(e) => result.push(e),
                ExpressionOrChr::Chr(chr) => {
                    return if chr == b')' {
                        Ok(Expression::Tuple(result))
                    } else {
                        unexpected_chr(chr)
                    }
                }
            }
        }
    }

    fn parse_string(&mut self) -> ParseResult<Expression> {
        let mut result = Vec::new();
        while let Some(chr) = self.chr() {
            match chr {
                b'\\' => {
                    self.seek()?;
                    if let Some(chr) = &self.chr() {
                        let chr = match chr {
                            b'\\' => b'\\',
                            b'"' => b'"',
                            _ => return unexpected_chr(*chr),
                        };
                        result.push(chr);
                        self.seek()?;
                    }
                }
                b'"' => {
                    self.seek()?;

                    return String::from_utf8(result)
                        .map(Expression::String)
                        .map_err(Error::Utf8Error);
                }
                _ => {
                    result.push(chr);
                    self.seek()?;
                }
            }
        }

        unexpected_eof()
    }

    fn parse_backquoted_string(&mut self) -> ParseResult<Expression> {
        let mut result = Vec::new();
        while let Some(chr) = self.chr() {
            match chr {
                b'\\' => {
                    self.seek()?;
                    if let Some(chr) = &self.chr() {
                        let chr = match chr {
                            b'\\' => b'\\',
                            b'`' => b'`',
                            _ => return unexpected_chr(*chr),
                        };
                        result.push(chr);
                        self.seek()?;
                    }
                }
                b'`' => {
                    self.seek()?;

                    return String::from_utf8(result)
                        .map(Expression::BackQuotedString)
                        .map_err(Error::Utf8Error);
                }
                _ => {
                    result.push(chr);
                    self.seek()?;
                }
            }
        }

        unexpected_eof()
    }

    fn parse_number(&mut self, initial: u8) -> ParseResult<Expression> {
        fn str_to_u32(n: u8) -> u32 {
            (n - b'0').into()
        }

        let mut result = str_to_u32(initial);

        while let Some(chr) = self.chr() {
            match chr {
                b'0'..=b'9' => {
                    self.seek()?;
                    result = result * 10 + str_to_u32(chr);
                }
                _ => break,
            }
        }

        Ok(Expression::Integer(result))
    }

    fn parse_literal(&mut self, initial: u8) -> ParseResult<Expression> {
        let mut result = vec![initial];

        while let Some(chr) = self.chr() {
            if !chr.is_ascii_alphanumeric() {
                break;
            }
            result.push(chr);
            self.seek()?;
        }

        String::from_utf8(result)
            .map(Expression::Literal)
            .map_err(Error::Utf8Error)
    }

    fn roll_up_and_get(&mut self) -> ParseResult<u8> {
        while let Some(chr) = self.chr() {
            if chr.is_ascii_whitespace() {
                self.seek()?;
            } else {
                return Ok(chr);
            }
        }

        unexpected_eof()
    }
}

const fn unexpected_eof<T>() -> Result<T, Error> {
    Err(Error::ParseError(ParseError::UnexpectedEOF))
}

const fn unexpected_chr<T>(chr: u8) -> ParseResult<T> {
    Err(Error::ParseError(ParseError::UnexpectedCharacter(chr)))
}

#[macro_export]
macro_rules! unwrap_expr {
    ($e:expr, $typ:path) => {
        match $e {
            $typ(tmp) => Some(tmp),
            _ => None,
        }
    };
}

#[macro_export]
macro_rules! get_rand {
    ($iter:expr, $typ:path, $when_none:expr, $when_unexpected:expr) => {
        if let Some(rand) = $iter.next() {
            if let Some(value) = unwrap_expr!(rand, $typ) {
                Ok(value)
            } else {
                $when_unexpected
            }
        } else {
            $when_none
        }
    };
}

#[macro_export]
macro_rules! parse_func {
    ($name:ident (|$($param_name:ident : $param_type:path),+| $generator:expr, $when_unexpected:expr, $when_insufficient:expr, $when_exceeded:expr) -> $rtype:ty) => {
        fn $name(mut rand: RandIter) -> $rtype {
            $(let $param_name = get_rand!(rand, $param_type, $when_insufficient, $when_unexpected)?;)+
            if let Some(_) = rand.next() {
                $when_exceeded
            } else {
                $generator
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::{
        sexp::{Expression, SExpParser},
        string_reader::StringReader,
    };
    use std::iter;

    #[inline]
    fn test_base(txt: &str, expected: Expression) {
        let txt = txt.as_bytes();
        let reader = StringReader::new(txt).unwrap().unwrap();
        let mut parser = SExpParser::new(reader);
        assert_eq!(expected, parser.parse_expression().unwrap());
    }

    #[test]
    fn parse_empty_tuple() {
        test_base(r"()", Expression::Tuple(vec![]));
    }

    #[test]
    fn parse_string() {
        let text = "TestString1234567890!@#$%^&*()_+|~";
        test_base(
            &format!("\"{}\"", text),
            Expression::String(text.to_string()),
        );
    }

    #[test]
    fn parse_backquoted_string() {
        let text = "TestString1234567890!@#$%^&*()_+|~";
        test_base(
            &format!("`{}`", text),
            Expression::BackQuotedString(text.to_string()),
        );
    }

    #[test]
    fn parse_int() {
        let i = 1234567890u32;
        test_base(&format!("{}", i), Expression::Integer(i));
    }

    #[test]
    fn parse_literal() {
        let text = "TestString1234567890gnirtStseT";
        test_base(&format!("{}", text), Expression::Literal(text.to_string()))
    }

    #[test]
    fn parse_complicated() {
        let text = r#"
(() 123 "string" `backquoted` literal)
"#;
        test_base(
            &text,
            Expression::Tuple(vec![
                Expression::Tuple(vec![]),
                Expression::Integer(123),
                Expression::String("string".to_string()),
                Expression::BackQuotedString("backquoted".to_string()),
                Expression::Literal("literal".to_string()),
            ]),
        )
    }

    #[test]
    fn parse_nested() {
        fn nest(n: usize) -> String {
            if n == 0 {
                String::new()
            } else {
                format!("({})", nest(n - 1))
            }
        }

        let i = 40;
        test_base(
            &nest(i),
            iter::repeat(()).take(i - 1).fold(Expression::Tuple(vec![]), |acc, _| {
                Expression::Tuple(vec![acc])
            }),
        );
    }
}
