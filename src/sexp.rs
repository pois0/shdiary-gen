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
    ParseError,
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
            _ => Err(Error::ParseError),
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
                _ => break,
            }
        }
        return Err(Error::ParseError);
    }

    fn parse_text_item(&mut self) -> ParseResult<TextItem> {
        let keyword = self.parse_keyword()?;
        match keyword.as_str() {
            "a" => self.parse_weblink(),
            "b" => self.parse_bold(),
            "p" => self.parse_post(),
            _ => Err(Error::ParseError),
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
                    _ => return Err(Error::ParseError),
                }
            } else {
                return Err(Error::ParseError);
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
                    _ => return Err(Error::ParseError),
                }
            } else {
                return Err(Error::ParseError);
            }
        };

        loop {
            if let Some(chr) = self.chr {
                match chr {
                    0x20 | 0x09 | 0x0a | 0x0c | 0x0d => {
                        self.seek().map_err(Error::IOError)?;
                    }
                    b')' => break,
                    _ => return Err(Error::ParseError),
                }
            } else {
                return Err(Error::ParseError);
            }
        }

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
                    _ => return Err(Error::ParseError),
                }
            } else {
                return Err(Error::ParseError);
            }
        };

        self.roll_up_until_end()?;

        Ok(TextItem::Bold(string))
    }

    fn parse_post(&mut self) -> ParseResult<TextItem> {
        let year = loop {
            if let Some(chr) = self.chr {
                match chr {
                    0x20 | 0x09 | 0x0a | 0x0c | 0x0d => {
                        self.seek().map_err(Error::IOError)?;
                    }
                    b'0'..=b'9' => {
                        break self.parse_number()?;
                    }
                    _ => return Err(Error::ParseError),
                }
            }
        };

        let month = loop {
            if let Some(chr) = self.chr {
                match chr {
                    0x20 | 0x09 | 0x0a | 0x0c | 0x0d => {
                        self.seek().map_err(Error::IOError)?;
                    }
                    b'0'..=b'9' => {
                        break self.parse_number()?;
                    }
                    _ => return Err(Error::ParseError),
                }
            }
        };

        let day = loop {
            if let Some(chr) = self.chr {
                match chr {
                    0x20 | 0x09 | 0x0a | 0x0c | 0x0d => {
                        self.seek().map_err(Error::IOError)?;
                    }
                    b'0'..=b'9' => {
                        break self.parse_number()?;
                    }
                    _ => return Err(Error::ParseError),
                }
            }
        };

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
                _ => break,
            }
        }

        return Err(Error::ParseError);
    }

    fn parse_list(&mut self) -> ParseResult<Item> {
        self.parse_exprs().map(Item::List)
    }

    fn parse_string(&mut self) -> ParseResult<String> {
        let mut result = Vec::new();
        loop {
            if let Some(chr) = self.chr {
                match chr {
                    0x5c => {
                        self.seek().map_err(Error::IOError)?;
                        if let Some(chr) = &self.chr {
                            let chr = match chr {
                                0x5c => 0x5c,
                                0x22 => 0x22,
                                _ => return Err(Error::ParseError),
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
            } else {
                return Err(Error::ParseError);
            }
        }
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
                _ => break,
            }
        }

        Err(Error::ParseError)
    }

    fn parse_keyword(&mut self) -> ParseResult<String> {
        let mut result = Vec::new();
        loop {
            if let Some(chr) = self.chr {
                if chr.is_ascii_whitespace() {
                    self.seek().map_err(Error::IOError)?;
                    return String::from_utf8(result).map_err(Error::Utf8Error);
                }
                result.push(chr);
                self.seek().map_err(Error::IOError)?;
            } else {
                return Err(Error::ParseError);
            }
        }
    }

    fn roll_up_until_start(&mut self) -> ParseResult<()> {
        loop {
            if let Some(chr) = self.chr {
                match chr {
                    0x20 | 0x09 | 0x0a | 0x0c | 0x0d => {
                        self.seek().map_err(Error::IOError)?;
                    }
                    b'(' => {
                        self.seek().map_err(Error::IOError)?;
                        break Ok(());
                    }
                    _ => break Err(Error::ParseError),
                }
            } else {
                break Err(Error::ParseError);
            }
        }
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
                _ => return Err(Error::ParseError),
            }
        }

        return Err(Error::ParseError);
    }

    fn roll_up_until_end(&mut self) -> ParseResult<()> {
        loop {
            if let Some(chr) = self.chr {
                match chr {
                    0x20 | 0x09 | 0x0a | 0x0c | 0x0d => {
                        self.seek().map_err(Error::IOError)?;
                    }
                    b')' => {
                        self.seek().map_err(Error::IOError)?;
                        break Ok(());
                    }
                    _ => break Err(Error::ParseError),
                }
            } else {
                break Err(Error::ParseError);
            }
        }
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
