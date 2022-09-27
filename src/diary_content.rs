use std::{
    io,
    string::FromUtf8Error,
};

use crate::get_rand;
use crate::parse_func;
use crate::unwrap_expr;
use crate::{
    new_sexp::{self, Application, Expression, RandIter},
};

macro_rules! parse_diary_func {
    ($name:ident (|$($param_name:ident : $param_type:path),+| $generator:expr) -> $rtype:ty) => {
        parse_func!(
            $name(
                |$($param_name : $param_type),+| $generator,
                illegal_element(),
                operand_mismatch(),
                operand_mismatch()
            ) -> ParseResult<$rtype>
        );
    };
}

macro_rules! get_rand_diary {
    ($iter:expr, $typ:path) => {
        get_rand!($iter, $typ, operand_mismatch(), illegal_element())
    };
}

#[derive(Clone, Debug)]
pub struct Document<T: Sized + Clone> {
    contents: Vec<Item<T>>,
}

pub type SourceDoucument = Document<String>;

impl<T: Sized + Clone> Document<T> {
    pub const fn new(contents: Vec<Item<T>>) -> Self {
        Document { contents }
    }

    pub const fn contents(self: &Self) -> &Vec<Item<T>> {
        &self.contents
    }

    pub fn into_contents(self) -> Vec<Item<T>> {
        self.contents
    }

    pub const fn empty() -> Self {
        Document { contents: vec![] }
    }
}

#[derive(Clone, Debug)]
pub enum Item<T: Sized + Clone> {
    Text(Text),
    List(Vec<Item<T>>),
    Header(String),
    Images(Images<T>),
}

pub type SourceItem = Item<String>;

pub type Text = Vec<TextItem>;

#[derive(Clone, Debug)]
pub enum TextItem {
    RawString(String),
    Bold(String),
    WebLink(WebLink),
    PostLink((u32, u32, u32)),
    Code(String),
}

#[derive(Clone, Debug)]
pub struct WebLink {
    pub title: String,
    pub href: String,
}

#[derive(Clone, Debug)]
pub struct Images<T: Sized + Clone> {
    pub title: String,
    pub items: Vec<ImageItem<T>>,
}

#[derive(Clone, Debug)]
pub struct ImageItem<T: Sized + Clone> {
    pub data: T,
    pub caption: Option<String>,
}

#[derive(Debug)]
pub enum Error {
    IOError(io::Error),
    Utf8Error(FromUtf8Error),
    ParseError(new_sexp::ParseError),
    SyntaxError(SyntaxError),
}

#[derive(Debug)]
pub enum SyntaxError {
    IllegalElement,
    MissingOperator,
    UnknownOperator(String),
    OperandMismatch,
}

pub type ParseResult<T> = Result<T, Error>;

pub fn parse_diary_content(expr: Expression) -> ParseResult<SourceDoucument> {
    match expr {
        Expression::Tuple(l) => parse_top_list(l).map(Document::new),
        _ => Err(Error::SyntaxError(SyntaxError::IllegalElement)),
    }
}

fn parse_top_list(list: Vec<Expression>) -> ParseResult<Vec<SourceItem>> {
    list.into_iter().map(parse_top_expr).collect()
}

fn parse_top_expr(expr: Expression) -> ParseResult<SourceItem> {
    match expr {
        Expression::Tuple(t) => {
            let app = Application::new(t);
            if let Some(rator) = app.rator() {
                let rand = app.into_rand_iter();
                match rator {
                    "h" | "header" => parse_header(rand),
                    "txt" | "text" => parse_text(rand),
                    "li" | "list" => parse_list(rand),
                    "img" | "image" => parse_image(rand),
                    _ => unknown_operator(rator.to_owned()),
                }
            } else {
                missing_operator()
            }
        }
        Expression::String(s) => Ok(Item::Text(vec![TextItem::RawString(s)])),
        _ => illegal_element(),
    }
}

parse_diary_func! {
    parse_header(
        |s: Expression::String| {
            Ok(Item::Header(s))
        }
    ) -> SourceItem
}

fn parse_text(rand: RandIter) -> ParseResult<SourceItem> {
    rand.map(parse_text_item).collect::<ParseResult<Text>>().map(SourceItem::Text)
}

fn parse_text_item(expr: Expression) -> ParseResult<TextItem> {
    match expr {
        Expression::Tuple(t) => {
            let app = Application::new(t);
            if let Some(rator) = app.rator() {
                let rand = app.into_rand_iter();
                match rator {
                    "a" => parse_weblink(rand),
                    "b" => parse_bold(rand),
                    "p" => parse_post(rand),
                    "code" => parse_code(rand),
                    _ => unknown_operator(rator.to_owned()),
                }
            } else {
                missing_operator()
            }
        }
        Expression::String(s) => Ok(TextItem::RawString(s.to_string())),
        _ => illegal_element(),
    }
}

parse_diary_func! {
    parse_weblink(
        |title: Expression::String, href: Expression::String| {
            Ok(TextItem::WebLink(WebLink { title, href }))
        }
    ) -> TextItem
}

parse_diary_func! {
    parse_bold(
        |txt: Expression::String| {
            Ok(TextItem::Bold(txt))
        }
    ) -> TextItem
}

parse_diary_func! {
    parse_post(
        |year: Expression::Integer, month: Expression::Integer, day: Expression::Integer| {
            Ok(TextItem::PostLink((year, month, day)))
        }
    ) -> TextItem
}

parse_diary_func! {
    parse_code(
        |s: Expression::String| {
            Ok(TextItem::Code(s))
        }
    ) -> TextItem
}

fn parse_list(rand: RandIter) -> ParseResult<SourceItem> {
    rand.map(parse_list_item).collect::<ParseResult<Vec<SourceItem>>>().map(SourceItem::List)
}

fn parse_list_item(expr: Expression) -> ParseResult<SourceItem> {
    match expr {
        Expression::Tuple(t) => {
            let app = Application::new(t);
            if let Some(rator) = app.rator() {
                let rand = app.into_rand_iter();
                match rator {
                    "txt" | "text" => parse_text(rand),
                    "li" | "list" => parse_list(rand),
                    "img" | "image" => parse_image(rand),
                    _ => unknown_operator(rator.to_owned()),
                }
            } else {
                missing_operator()
            }
        }
        Expression::String(s) => Ok(Item::Text(vec![TextItem::RawString(s)])),
        _ => illegal_element(),
    }
}

fn parse_image(rand: RandIter) -> ParseResult<SourceItem> {
    let title = get_rand_diary!(rand, Expression::String)?;

    let items = rand.map(parse_image_items).collect::<ParseResult<Vec<ImageItem<String>>>>()?;

    Ok(Item::Images( Images { title, items }))
}

fn parse_image_items(expr: Expression) -> ParseResult<ImageItem<String>> {
    match expr {
         Expression::Tuple(t) => {
            let mut tuple_iter = t.into_iter();
            let path = get_rand!(tuple_iter, Expression::String, operand_mismatch(), illegal_element())?;
            let caption = tuple_iter.next().map_or(Ok(None), |e| unwrap_expr!(e, Expression::String).ok_or_else(|| Error::SyntaxError(SyntaxError::IllegalElement)).map(Some))?;
            Ok(ImageItem { data: path, caption })
         }
         _ => illegal_element(),
    }
}

const fn no_more_rand(rand: RandIter) -> ParseResult<()> {
    if let Some(_) = rand.next() {
        Err(Error::SyntaxError(SyntaxError::OperandMismatch))
    } else {
        Ok(())
    }
}

const fn missing_operator<T>() -> ParseResult<T> {
    Err(Error::SyntaxError(SyntaxError::IllegalElement))
}

const fn illegal_element<T>() -> ParseResult<T> {
    Err(Error::SyntaxError(SyntaxError::IllegalElement))
}

const fn unknown_operator<T>(name: String) -> ParseResult<T> {
    Err(Error::SyntaxError(SyntaxError::UnknownOperator(name)))
}

const fn operand_mismatch<T>() -> ParseResult<T> {
    Err(Error::SyntaxError(SyntaxError::OperandMismatch))
}
