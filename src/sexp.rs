use std::io::{self, Bytes, Read};
use std::string::{FromUtf8Error, String};

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
type IOResult<T> = Result<T, io::Error>;

struct ParseCtx<R: Read> {
    bytes: Bytes<R>,
    chr: Option<u8>,
}

impl<R: Read> ParseCtx<R> {
    fn new(raw_read: R) -> Option<IOResult<Self>> {
        let mut bytes = raw_read.bytes();
        bytes.next().map(|chr| {
            chr.map(|chr| Self {
                bytes,
                chr: Some(chr),
            })
        })
    }

    fn seek(&mut self) -> IOResult<()> {
        self.chr = match self.bytes.next() {
            Some(res) => {
                let chr = res?;
                Some(chr)
            }
            None => None,
        };
        Ok(())
    }

    fn parse_root(&mut self) -> ParseResult<Document> {
        self.roll_up_until_start()?;
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
        while let Some(chr) = self.chr {
            match chr {
                0x20 | 0x09 | 0x0a | 0x0c | 0x0d => {
                    self.seek().map_err(Error::IOError)?;
                }
                b'(' => {
                    self.seek().map_err(Error::IOError)?;
                    let item = self.parse_text_item()?;
                    result.push(item);
                }
                b')' => {
                    self.seek().map_err(Error::IOError)?;
                    return Ok(Item::Text(result));
                }
                b'"' => {
                    self.seek().map_err(Error::IOError)?;
                    let text = self.parse_string()?;
                    result.push(TextItem::RawString(text));
                }
                _ => {
                    return unexpected_chr(chr)
                }
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
            _ => {
                println!("yooyo!!");
                unexpected_keyword(keyword)
            }
        }
    }

    fn parse_weblink(&mut self) -> ParseResult<TextItem> {
        let title = loop {
            if let Some(chr) = self.chr {
                match chr {
                    0x20 | 0x09 | 0x0a | 0x0c | 0x0d => {
                        self.seek().map_err(Error::IOError)?;
                    }
                    b'"' => {
                        self.seek().map_err(Error::IOError)?;
                        let string = self.parse_string()?;
                        break string;
                    }
                    _ => {
                        return unexpected_chr(chr)
                    }
                }
            } else {
                return unexpected_eof()
            }
        };

        let href = loop {
            if let Some(chr) = self.chr {
                match chr {
                    0x20 | 0x09 | 0x0a | 0x0c | 0x0d => {
                        self.seek().map_err(Error::IOError)?;
                    }
                    b'"' => {
                        self.seek().map_err(Error::IOError)?;
                        let string = self.parse_string()?;
                        break string;
                    }
                    _ => {
                        return unexpected_chr(chr)
                    }
                }
            } else {
                return unexpected_eof()
            }
        };

        self.roll_up_until_end()?;

        Ok(TextItem::WebLink(WebLink { title, href }))
    }

    fn parse_bold(&mut self) -> ParseResult<TextItem> {
        let string = loop {
            if let Some(chr) = self.chr {
                match chr {
                    0x20 | 0x09 | 0x0a | 0x0c | 0x0d => {
                        self.seek().map_err(Error::IOError)?;
                    }
                    b'"' => {
                        self.seek().map_err(Error::IOError)?;
                        let string = self.parse_string()?;
                        break string;
                    }
                    _ => {
                        return unexpected_chr(chr)
                    }
                }
            } else {
                return unexpected_eof()
            }
        };

        self.roll_up_until_end()?;

        Ok(TextItem::Bold(string))
    }

    fn parse_post(&mut self) -> ParseResult<TextItem> {
        let year = self.parse_number()?;

        let month = self.parse_number()?;

        let day = self.parse_number()?;

        self.roll_up_until_end()?;

        Ok(TextItem::PostLink((year, month, day)))
    }

    fn parse_header(&mut self) -> ParseResult<Item> {
        while let Some(chr) = self.chr {
            match chr {
                0x20 | 0x09 | 0x0a | 0x0c | 0x0d => {
                    self.seek().map_err(Error::IOError)?;
                }
                b'"' => {
                    self.seek().map_err(Error::IOError)?;
                    let string = self.parse_string()?;
                    self.roll_up_until_end()?;
                    return Ok(Item::Header(string));
                }
                _ => {
                    return unexpected_chr(chr)
                }
            }
        }

        unexpected_eof()
    }

    fn parse_list(&mut self) -> ParseResult<Item> {
        self.parse_exprs().map(Item::List)
    }

    fn parse_string(&mut self) -> ParseResult<String> {
        let mut result = Vec::new();
        while let Some(chr) = self.chr {
            match chr {
                0x5c => {
                    self.seek().map_err(Error::IOError)?;
                    if let Some(chr) = &self.chr {
                        let chr = match chr {
                            0x5c => 0x5c,
                            0x22 => 0x22,
                            _ => {
                                return unexpected_chr(*chr)
                            }
                        };
                        result.push(chr);
                    }
                }
                0x22 => {
                    self.seek().map_err(Error::IOError)?;

                    return String::from_utf8(result).map_err(Error::Utf8Error);
                }
                _ => {
                    result.push(chr);
                    self.seek().map_err(Error::IOError)?;
                }
            }
        }

        unexpected_eof()
    }

    fn parse_number(&mut self) -> ParseResult<u16> {
        let mut result = 0u16;
        while let Some(chr) = self.chr {
            match chr {
                b'0'..=b'9' => {
                    self.seek().map_err(Error::IOError)?;
                    result = result * 10 + u16::from(chr - b'0');
                }
                0x20 | 0x09 | 0x0a | 0x0c | 0x0d => {
                    self.seek().map_err(Error::IOError)?;
                    return Ok(result);
                }
                _ => {
                    return unexpected_chr(chr)
                }
            }
        }

        unexpected_eof()
    }

    fn parse_keyword(&mut self) -> ParseResult<String> {
        let mut result = Vec::new();

        while let Some(chr) = self.chr {
            if chr.is_ascii_whitespace() {
                self.seek().map_err(Error::IOError)?;
                return String::from_utf8(result).map_err(Error::Utf8Error);
            }
            result.push(chr);
            self.seek().map_err(Error::IOError)?;
        }
        
        unexpected_eof()
    }

    fn roll_up_until_start(&mut self) -> ParseResult<()> {
        if let Some(chr) = self.chr {
            match chr {
                0x20 | 0x09 | 0x0a | 0x0c | 0x0d => {
                    self.seek().map_err(Error::IOError)?;
                }
                b'(' => {
                    self.seek().map_err(Error::IOError)?;
                    return Ok(());
                }
                _ => {
                    return unexpected_chr(chr)
                }
            }
        }
        unexpected_eof()
    }

    fn parse_exprs(&mut self) -> ParseResult<Vec<Item>> {
        let mut result = Vec::new();

        while let Some(chr) = self.chr {
            match chr {
                0x20 | 0x09 | 0x0a | 0x0c | 0x0d => {
                    self.seek().map_err(Error::IOError)?;
                }
                b'(' => {
                    self.seek().map_err(Error::IOError)?;
                    let item = self.parse_expr()?;
                    result.push(item);
                }
                b')' => {
                    self.seek().map_err(Error::IOError)?;
                    return Ok(result);
                }
                b'"' => {
                    self.seek().map_err(Error::IOError)?;
                    let text = self.parse_string()?;
                    result.push(Item::Text(vec![TextItem::RawString(text)]));
                }
                _ => {
                    return unexpected_chr(chr)
                }
            }
        }

        unexpected_eof()
    }

    fn roll_up_until_end(&mut self) -> ParseResult<()> {
        while let Some(chr) = self.chr {
            match chr {
                0x20 | 0x09 | 0x0a | 0x0c | 0x0d => {
                    self.seek().map_err(Error::IOError)?;
                }
                b')' => {
                    self.seek().map_err(Error::IOError)?;
                    return Ok(())
                }
                _ => {
                    return unexpected_chr(chr)
                }
            }
        }

        unexpected_eof()
    }
}

pub fn parse<R: Read>(read: R) -> ParseResult<Document> {
    if let Some(ctx_result) = ParseCtx::new(read) {
        let mut ctx = ctx_result.map_err(Error::IOError)?;
        ctx.parse_root()
    } else {
        Ok(Document::empty())
    }
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
