use std::io::{self, Read};
use std::string::{FromUtf8Error, String};

use crate::string_reader::StringReader;
use crate::roll_up_until;

#[derive(Clone, Debug)]
pub struct Document {
    contents: Vec<Item>,
}

impl Document {
    pub const fn new(contents: Vec<Item>) -> Self {
        Document { contents }
    }

    pub const fn contents(self: &Self) -> &Vec<Item> {
        &self.contents
    }

    pub const fn empty() -> Self {
        Document { contents: vec![] }
    }
}

#[derive(Clone, Debug)]
pub enum Item {
    Text(Text),
    List(Vec<Item>),
    Header(String),
}

pub type Text = Vec<TextItem>;

#[derive(Clone, Debug)]
pub enum TextItem {
    RawString(String),
    Bold(String),
    WebLink(WebLink),
    PostLink((u16, u16, u16)),
}

#[derive(Clone, Debug)]
pub struct WebLink {
    pub title: String,
    pub href: String,
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
}

pub type ParseResult<T> = Result<T, Error>;

struct ParseCtx<R: Read> {
    reader: StringReader<R>,
}

impl<R: Read> ParseCtx<R> {

    fn new(reader: StringReader<R>) -> Self {
        Self { reader }
    }

    fn chr(&self) -> Option<u8> {
        self.reader.chr()
    }

    fn seek(&mut self) -> ParseResult<()> {
        self.reader.seek().map_err(Error::IOError)
    }

    fn parse_root(&mut self) -> ParseResult<Document> {
        roll_up_until!(self, b'(')?;
        self.seek()?;
        self.parse_exprs().map(Document::new)
    }

    fn parse_expr(&mut self) -> ParseResult<Item> {
        let keyword = self.parse_keyword()?;
        match keyword.as_str() {
            "txt" | "text" => self.parse_text(),
            "li" | "list" => self.parse_list(),
            "h" | "header" => self.parse_header(),
            _ => unexpected_keyword(keyword),
        }
    }

    fn parse_text(&mut self) -> ParseResult<Item> {
        let mut result = Vec::new();
        while let Some(chr) = self.chr() {
            match chr {
                0x20 | 0x09 | 0x0a | 0x0c | 0x0d => {
                    self.seek()?;
                }
                b'(' => {
                    self.seek()?;
                    let item = self.parse_text_item()?;
                    result.push(item);
                }
                b')' => {
                    self.seek()?;
                    return Ok(Item::Text(result));
                }
                b'"' => {
                    self.seek()?;
                    let text = self.parse_string()?;
                    result.push(TextItem::RawString(text));
                }
                _ => return unexpected_chr(chr),
            }
        }

        unexpected_eof()
    }

    fn parse_text_item(&mut self) -> ParseResult<TextItem> {
        let keyword = self.parse_keyword()?;
        match keyword.as_str() {
            "a" => self.parse_weblink(),
            "b" => self.parse_bold(),
            "p" => self.parse_post(),
            _ => unexpected_keyword(keyword),
        }
    }

    fn parse_weblink(&mut self) -> ParseResult<TextItem> {
        let title = self.expect_string()?;
        let href = self.expect_string()?;
        roll_up_until!(self, b')')?;
        self.seek()?;

        Ok(TextItem::WebLink(WebLink { title, href }))
    }

    fn parse_bold(&mut self) -> ParseResult<TextItem> {
        let string = self.expect_string()?;

        roll_up_until!(self, b')')?;
        self.seek()?;

        Ok(TextItem::Bold(string))
    }

    fn parse_post(&mut self) -> ParseResult<TextItem> {
        let year = self.expect_number()?;

        let month = self.expect_number()?;

        let day = self.expect_number()?;

        roll_up_until!(self, b')')?;
        self.seek()?;

        Ok(TextItem::PostLink((year, month, day)))
    }

    fn parse_header(&mut self) -> ParseResult<Item> {
        roll_up_until!(self, b'"', {
            self.seek()?;
            let string = self.parse_string()?;
            roll_up_until!(self, b')')?;
            self.seek()?;
            Ok(Item::Header(string))
        })
    }

    fn parse_list(&mut self) -> ParseResult<Item> {
        self.parse_exprs().map(Item::List)
    }

    fn expect_string(&mut self) -> ParseResult<String> {
        roll_up_until!(self, b'"')?;
        self.seek()?;
        self.parse_string()
    }

    fn parse_string(&mut self) -> ParseResult<String> {
        let mut result = Vec::new();
        while let Some(chr) = self.chr() {
            match chr {
                0x5c => {
                    self.seek()?;
                    if let Some(chr) = &self.chr() {
                        let chr = match chr {
                            0x5c => 0x5c,
                            0x22 => 0x22,
                            _ => return unexpected_chr(*chr),
                        };
                        result.push(chr);
                    }
                }
                0x22 => {
                    self.seek()?;

                    return String::from_utf8(result).map_err(Error::Utf8Error);
                }
                _ => {
                    result.push(chr);
                    self.seek()?;
                }
            }
        }

        unexpected_eof()
    }

    fn expect_number(&mut self) -> ParseResult<u16> {
        roll_up_until!(self, b'0'..=b'9')?;
        self.parse_number()
    }

    fn parse_number(&mut self) -> ParseResult<u16> {
        let mut result = 0u16;
        while let Some(chr) = self.chr() {
            match chr {
                b'0'..=b'9' => {
                    self.seek()?;
                    result = result * 10 + u16::from(chr - b'0');
                }
                _ => {
                    return Ok(result);
                }
            }
        }

        unexpected_eof()
    }

    fn parse_keyword(&mut self) -> ParseResult<String> {
        let mut result = Vec::new();

        while let Some(chr) = self.chr() {
            if chr.is_ascii_whitespace() {
                self.seek()?;
                return String::from_utf8(result).map_err(Error::Utf8Error);
            }
            result.push(chr);
            self.seek()?;
        }

        unexpected_eof()
    }

    fn parse_exprs(&mut self) -> ParseResult<Vec<Item>> {
        let mut result = Vec::new();

        while let Some(chr) = self.chr() {
            match chr {
                0x20 | 0x09 | 0x0a | 0x0c | 0x0d => {
                    self.seek()?;
                }
                b'(' => {
                    self.seek()?;
                    let item = self.parse_expr()?;
                    result.push(item);
                }
                b')' => {
                    self.seek()?;
                    return Ok(result);
                }
                b'"' => {
                    self.seek()?;
                    let text = self.parse_string()?;
                    result.push(Item::Text(vec![TextItem::RawString(text)]));
                }
                _ => return unexpected_chr(chr),
            }
        }

        unexpected_eof()
    }
}

pub fn parse<R: Read>(read: R) -> ParseResult<Document> {
    let reader = StringReader::new(read).map_err(Error::IOError)?;
    reader.map_or(Ok(Document::empty()), |reader| {
        ParseCtx::new(reader).parse_root()
    })
}

fn unexpected_eof<T>() -> Result<T, Error> {
    Err(Error::ParseError(ParseError::UnexpectedEOF))
}

fn unexpected_chr<T>(chr: u8) -> ParseResult<T> {
    Err(Error::ParseError(ParseError::UnexpectedCharacter(chr)))
}

fn unexpected_keyword<T>(keyword: String) -> ParseResult<T> {
    Err(Error::ParseError(ParseError::UnknownKeyword(keyword)))
}

#[macro_export]
macro_rules! roll_up_until {
    ($reader: expr, $cond: pat) => {
        '__roll_up_until_lablel: loop {
            if let Some(chr) = $reader.chr() {
                match chr {
                    0x20 | 0x09 | 0x0a | 0x0c | 0x0d => {
                        $reader.seek()?;
                    }
                    $cond => {
                        break '__roll_up_until_lablel Ok(())
                    }
                    _ => {
                        break '__roll_up_until_lablel unexpected_chr(chr)
                    }
                }
            } else {
                break '__roll_up_until_lablel unexpected_eof()
            }
        }
    };
    ($reader: expr, $cond: pat, $then: block) => {
        '__roll_up_until_lablel: loop {
            if let Some(chr) = $reader.chr() {
                match chr {
                    0x20 | 0x09 | 0x0a | 0x0c | 0x0d => {
                        $reader.seek()?;
                    }
                    $cond => {
                        break '__roll_up_until_lablel ($then)
                    }
                    _ => {
                        break '__roll_up_until_lablel unexpected_chr(chr)
                    }
                }
            } else {
                break '__roll_up_until_lablel unexpected_eof()
            }
        }
    };
}
