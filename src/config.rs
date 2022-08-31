use std::{io::{Read, self}, string::FromUtf8Error, collections::HashMap};

use crate::string_reader::StringReader;

pub struct ParseCtx<R: Read> {
    reader: StringReader<R>,
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
    UnknownKeyword(String),
    EmptyKey,
}

pub type ParseResult<T> = Result<T, Error>;

impl<R: Read> ParseCtx<R> {
    fn new(reader: StringReader<R>) -> Self {
        Self { reader }
    }

    const fn chr(&self) -> Option<u8> {
        self.reader.chr()
    }

    fn seek(&mut self) -> ParseResult<()> {
        self.reader.seek().map_err(Error::IOError)
    }

    fn parse_root(&mut self) -> ParseResult<HashMap<String, String>> {
        let mut result = HashMap::new();

        loop {
            let is_eof = !self.trim_space()?;
            if is_eof {
                break
            }
            self.parse_key()?;
            self.trim_space_until_value()?;
            self.parse_value()?;
            self.trim_space_until_break_line()?;
        }

        Ok(result)
    }

    fn parse_key(&mut self) -> ParseResult<String> {
        let mut result = Vec::new();
        while let Some(chr) = self.chr() {
            match chr {
                0x20 | 0x09 | 0x0c | 0x0d | b'=' => {
                    return if result.is_empty() {
                        Err(Error::ParseError(ParseError::EmptyKey))
                    } else {
                        String::from_utf8(result).map_err(Error::Utf8Error)
                    }
                }
                0x0a => {
                    return unexpected_chr(chr)
                }
                _ => {
                    result.push(chr);
                    self.seek()?;
                }
            }
        }

        unexpected_eof()
    }

    fn parse_value(&mut self) -> ParseResult<String> {
        let mut result = Vec::new();
        while let Some(chr) = self.chr() {
            match chr {
                0x20 | 0x09 | 0x0a | 0x0c | 0x0d => {
                    break
                }
                _ => {
                    result.push(chr);
                    self.seek()?;
                }
            }
        }

        if result.is_empty() {
            Err(Error::ParseError(ParseError::EmptyKey))
        } else {
            String::from_utf8(result).map_err(Error::Utf8Error)
        }
    }

    fn trim_space(&mut self) -> ParseResult<bool> {
        while let Some(chr) = self.chr() {
            match chr {
                0x20 | 0x09 | 0x0a | 0x0c | 0x0d => {
                    self.seek()?;
                }
                _ => {
                    return Ok(true)
                }
            }
        }

        Ok(false)
    }

    fn trim_space_until_break_line(&mut self) -> ParseResult<bool> {
        while let Some(chr) = self.chr() {
            match chr {
                0x20 | 0x09 | 0x0c | 0x0d => {
                    self.seek()?;
                }
                0x0a => {
                    self.seek()?;
                    return Ok(true)
                }
                _ => {
                    return unexpected_chr(chr)
                }
            }
        }

        Ok(false)
    }
}

fn unexpected_eof<T>() -> Result<T, Error> {
    Err(Error::ParseError(ParseError::UnexpectedEOF))
}

fn unexpected_chr<T>(chr: u8) -> ParseResult<T> {
    Err(Error::ParseError(ParseError::UnexpectedCharacter(chr)))
}
