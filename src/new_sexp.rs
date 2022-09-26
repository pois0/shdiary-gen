use std::{
    io::{self, Read},
    string::FromUtf8Error,
};

use crate::string_reader::StringReader;

#[derive(Debug, Clone)]
pub enum Expression {
    Tuple(Vec<Expression>),
    Literal(String),
    String(String),
    Integer(u32),
}

#[derive(Clone, Copy)]
pub struct Application<'a> {
    raw: &'a [Expression],
}

impl<'a> Application<'a> {
    pub fn new(tuple: &'a Vec<Expression>) -> Self {
        Self {
            raw: tuple.as_slice(),
        }
    }

    pub fn rator(&self) -> Option<&str> {
        self.raw.get(0).and_then(|it| match it {
            Expression::Literal(l) => Some(l.as_str()),
            _ => None,
        })
    }

    pub fn rand(&self) -> Option<&'a [Expression]> {
        self.raw.get(1..)
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

struct ParseCtx<R: Read> {
    reader: StringReader<R>,
}

enum ExpressionOrChr {
    Expression(Expression),
    Chr(u8),
}

impl<R: Read> ParseCtx<R> {
    const fn new(reader: StringReader<R>) -> Self {
        Self { reader }
    }

    const fn chr(&self) -> Option<u8> {
        self.reader.chr()
    }

    fn seek(&mut self) -> ParseResult<()> {
        self.reader.seek().map_err(Error::IOError)
    }

    fn parse_expression(&mut self) -> ParseResult<ExpressionOrChr> {
        let chr = self.roll_up_and_get()?;
        self.seek()?;
        match chr {
            b'(' => self.parse_tuple().map(ExpressionOrChr::Expression),
            b'"' => self.parse_string().map(ExpressionOrChr::Expression),
            b'0'..=b'9' => self.parse_number(chr).map(ExpressionOrChr::Expression),
            b'a'..=b'z' | b'A'..=b'Z' => self.parse_literal(chr).map(ExpressionOrChr::Expression),
            _ => Ok(ExpressionOrChr::Chr(chr)),
        }
    }

    fn parse_tuple(&mut self) -> ParseResult<Expression> {
        let mut result = Vec::new();

        loop {
            let node = self.parse_expression()?;
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
                _ => {
                    return Ok(Expression::Integer(result));
                }
            }
        }

        unexpected_eof()
    }

    fn parse_literal(&mut self, initial: u8) -> ParseResult<Expression> {
        let mut result = vec![initial];

        while let Some(chr) = self.chr() {
            if chr.is_ascii_whitespace() {
                return String::from_utf8(result)
                    .map(Expression::Literal)
                    .map_err(Error::Utf8Error);
            }
            result.push(chr);
            self.seek()?;
        }

        unexpected_eof()
    }

    fn roll_up_and_get(&mut self) -> ParseResult<u8> {
        while let Some(chr) = self.chr() {
            match chr {
                0x20 | 0x09 | 0x0a | 0x0c | 0x0d => {
                    self.seek()?;
                }
                _ => return Ok(chr),
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
