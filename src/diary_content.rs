use crate::sexp::{Expression, RandIter};
use crate::syntax_error::ParseResult;
use crate::syntax_error::{illegal_element, Error};
use crate::{get_rand, get_rand_diary, match_keyword, parse_diary_func, unwrap_expr};

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

pub fn parse_diary_content(expr: Expression) -> ParseResult<SourceDoucument> {
    match expr {
        Expression::Tuple(l) => parse_top_list(l).map(Document::new),
        _ => illegal_element(),
    }
}

fn parse_top_list(list: Vec<Expression>) -> ParseResult<Vec<SourceItem>> {
    list.into_iter().map(parse_top_expr).collect()
}

fn parse_top_expr(expr: Expression) -> ParseResult<SourceItem> {
    match expr {
        Expression::Tuple(t) => {
            match_keyword! { t, |rand| {
                "h" | "header" => parse_header(rand),
                "txt" | "text" => parse_text(rand),
                "li" | "list" => parse_list(rand),
                "img" | "image" => parse_image(rand)
            }}
        }
        Expression::String(s) => Ok(Item::Text(vec![TextItem::RawString(s)])),
        Expression::BackQuotedString(s) => Ok(Item::Text(vec![TextItem::Code(s)])),
        _ => illegal_element(),
    }
}

parse_diary_func! {
    parse_header(|s: Expression::String| Ok(Item::Header(s))) -> SourceItem
}

fn parse_text(rand: RandIter) -> ParseResult<SourceItem> {
    rand.map(parse_text_item)
        .collect::<ParseResult<Text>>()
        .map(SourceItem::Text)
}

fn parse_text_item(expr: Expression) -> ParseResult<TextItem> {
    match expr {
        Expression::Tuple(t) => {
            match_keyword! { t, |rand| {
                "a" => parse_weblink(rand),
                "b" => parse_bold(rand),
                "p" => parse_post(rand),
                "code" => parse_code(rand)
            }}
        }
        Expression::String(s) => Ok(TextItem::RawString(s)),
        Expression::BackQuotedString(s) => Ok(TextItem::Code(s)),
        _ => illegal_element(),
    }
}

parse_diary_func! {
    parse_weblink(|title: Expression::String, href: Expression::String| {
        Ok(TextItem::WebLink(WebLink { title, href }))
    }) -> TextItem
}

parse_diary_func! {
    parse_bold(|txt: Expression::String| Ok(TextItem::Bold(txt))) -> TextItem
}

parse_diary_func! {
    parse_post(|year: Expression::Integer, month: Expression::Integer, day: Expression::Integer| {
        Ok(TextItem::PostLink((year, month, day)))
    }) -> TextItem
}

parse_diary_func! {
    parse_code(|s: Expression::String| Ok(TextItem::Code(s))) -> TextItem
}

fn parse_list(rand: RandIter) -> ParseResult<SourceItem> {
    rand.map(parse_list_item)
        .collect::<ParseResult<Vec<SourceItem>>>()
        .map(SourceItem::List)
}

fn parse_list_item(expr: Expression) -> ParseResult<SourceItem> {
    match expr {
        Expression::Tuple(t) => {
            match_keyword! (t, |rand| {
                    "txt" | "text" => parse_text(rand),
                    "li" | "list" => parse_list(rand),
                    "img" | "image" => parse_image(rand)
            })
        }
        Expression::String(s) => Ok(Item::Text(vec![TextItem::RawString(s)])),
        _ => illegal_element(),
    }
}

fn parse_image(mut rand: RandIter) -> ParseResult<SourceItem> {
    let title = get_rand_diary!(&mut rand, Expression::String)?;

    let items = rand
        .map(parse_image_items)
        .collect::<ParseResult<Vec<ImageItem<String>>>>()?;

    Ok(Item::Images(Images { title, items }))
}

fn parse_image_items(expr: Expression) -> ParseResult<ImageItem<String>> {
    match expr {
        Expression::Tuple(t) => {
            let mut tuple_iter = t.into_iter();
            let path = get_rand_diary!(tuple_iter, Expression::String)?;
            let caption = tuple_iter.next().map_or(Ok(None), |e| {
                unwrap_expr!(e, Expression::String)
                    .ok_or(Error::IllegalElement)
                    .map(Some)
            })?;
            Ok(ImageItem {
                data: path,
                caption,
            })
        }
        _ => illegal_element(),
    }
}
