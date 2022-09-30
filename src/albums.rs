use crate::{
    date::Date,
    get_rand_diary, match_keyword_mut,
    sexp::Expression,
    syntax_error::{illegal_element, Error, ParseResult},
    unwrap_expr,
};

#[derive(Debug, Clone)]
pub struct AlbumIndex(pub Vec<Artist>);

#[derive(Debug, Clone, Eq, Ord)]
pub struct Artist {
    name: String,
    albums: Vec<Album>,
}

#[derive(Debug, Clone, Eq, Ord)]
pub struct Album {
    name: String,
    published_at: Date,
    link_to_diary: Option<Date>,
}

impl AlbumIndex {
    const fn new(artists: Vec<Artist>) -> Self {
        Self(artists)
    }
}

impl Artist {
    const fn new(name: String, albums: Vec<Album>) -> Self {
        Self { name, albums }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn albums(&self) -> &[Album] {
        self.albums.as_slice()
    }
}

impl PartialEq for Artist {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl PartialOrd for Artist {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.name.partial_cmp(&other.name)
    }
}

impl Album {
    const fn new(name: String, published_at: Date, link_to_diary: Option<Date>) -> Self {
        Self {
            name,
            published_at,
            link_to_diary,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn link_to_diary(&self) -> &Option<Date> {
        &self.link_to_diary
    }
}

impl PartialEq for Album {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl PartialOrd for Album {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.published_at.partial_cmp(&other.published_at)
    }
}

pub fn parse_albums(expr: Expression) -> ParseResult<AlbumIndex> {
    match expr {
        Expression::Tuple(l) => {
            let mut artists = parse_top_list(l)?;
            artists.sort();
            Ok(AlbumIndex::new(artists))
        }
        _ => illegal_element(),
    }
}

fn parse_top_list(list: Vec<Expression>) -> ParseResult<Vec<Artist>> {
    list.into_iter().map(parse_artist).collect()
}

fn parse_artist(expr: Expression) -> ParseResult<Artist> {
    let l = unwrap_expr!(expr, Expression::Tuple).ok_or(Error::IllegalElement)?;
    match_keyword_mut! { l, |rand| {
        "artist" => {
            let name = get_rand_diary!(rand, Expression::String)?;
            let mut albums = rand.map(parse_album).collect::<ParseResult<Vec<Album>>>()?;
            albums.sort();
            Ok(Artist::new(name, albums))
        }
    }}
}

fn parse_album(expr: Expression) -> ParseResult<Album> {
    let l = unwrap_expr!(expr, Expression::Tuple).ok_or(Error::IllegalElement)?;
    match_keyword_mut! { l, |rand| {
        "album" => {
            let name = get_rand_diary!(rand, Expression::String)?;
            let published_at = get_rand_diary!(rand, Expression::Tuple).and_then(parse_date)?;
            let link_to_diary = match get_rand_diary!(rand, Expression::Tuple) {
                Ok(l) => parse_date(l).map(Some),
                Err(Error::OperandMismatch) => Ok(None),
                Err(err) => Err(err)
            }?;
            Ok(Album::new(name, published_at, link_to_diary))
        }
    }}
}

fn parse_date(expr: Vec<Expression>) -> ParseResult<Date> {
    let mut iter = expr.into_iter();
    let year = get_rand_diary!(iter, Expression::Integer)?;
    let month = get_rand_diary!(iter, Expression::Integer)?;
    let day = get_rand_diary!(iter, Expression::Integer)?;
    let date = Date::new(year, month, day).unwrap();
    Ok(date)
}
