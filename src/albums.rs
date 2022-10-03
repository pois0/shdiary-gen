use crate::{
    date::Date,
    get_rand_diary, match_keyword, match_keyword_mut,
    sexp::{Expression, RandIter},
    syntax_error::{illegal_element, Error, ParseResult},
    unwrap_expr,
};

#[derive(Debug, Clone)]
pub struct AlbumIndex(pub Vec<Artist>);

#[derive(Debug, Clone)]
pub struct Artist {
    name: String,
    albums: AlbumList,
}

#[derive(Debug, Clone)]
pub struct AlbumList {
    studio_album: Vec<Album>,
    live_album: Vec<Album>,
    studio_and_live: Vec<Album>,
    compilation: Vec<Album>,
    concert: Vec<Album>,
}

#[derive(Debug, Clone, Eq)]
pub struct Album {
    name: String,
    published_at: Date,
    link_to_diary: Option<Date>,
}

#[derive(Clone, Copy)]
pub enum AlbumKind {
    StudioAlbum,
    LiveAlbum,
    StudioAndLive,
    Compilation,
    Concert,
}

impl AlbumIndex {
    const fn new(artists: Vec<Artist>) -> Self {
        Self(artists)
    }
}

impl Artist {
    const fn new(name: String, albums: AlbumList) -> Self {
        Self { name, albums }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn albums(&self) -> &AlbumList {
        &self.albums
    }
}

impl PartialEq for Artist {
    fn eq(&self, other: &Self) -> bool {
        self.name.eq(&other.name)
    }
}

impl Eq for Artist {}

impl PartialOrd for Artist {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Artist {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.albums
            .len()
            .cmp(&other.albums.len())
            .reverse()
            .then_with(|| self.name.cmp(&other.name))
    }
}

impl AlbumList {
    const fn new() -> Self {
        Self {
            studio_album: vec![],
            live_album: vec![],
            studio_and_live: vec![],
            compilation: vec![],
            concert: vec![],
        }
    }

    fn sort(&mut self) {
        self.studio_album.sort();
        self.live_album.sort();
        self.studio_and_live.sort();
        self.compilation.sort();
        self.concert.sort();
    }

    pub fn len(&self) -> usize {
        self.studio_album.len()
            + self.live_album.len()
            + self.studio_and_live.len()
            + self.compilation.len()
            + self.concert.len()
    }

    pub fn studio_album(&self) -> &[Album] {
        self.studio_album.as_slice()
    }

    pub fn live_album(&self) -> &[Album] {
        self.live_album.as_slice()
    }

    pub fn studio_and_live(&self) -> &[Album] {
        self.studio_and_live.as_slice()
    }

    pub fn compilation(&self) -> &[Album] {
        self.compilation.as_slice()
    }

    pub fn live(&self) -> &[Album] {
        self.concert.as_slice()
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
        Some(self.cmp(other))
    }
}

impl Ord for Album {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.published_at.cmp(&other.published_at)
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
                let mut album_list = AlbumList::new();
                for album_expr in rand {
                    let (kind, album) = parse_album(album_expr)?;
                    (match kind {
                        AlbumKind::StudioAlbum => &mut album_list.studio_album,
                        AlbumKind::LiveAlbum => &mut album_list.live_album,
                        AlbumKind::StudioAndLive => &mut album_list.studio_and_live,
                        AlbumKind::Compilation => &mut album_list.compilation,
                        AlbumKind::Concert => &mut album_list.concert
                    }).push(album);
                }
                album_list.sort();
                Ok(Artist::new(name, album_list))
            }
        }}
}

fn parse_album(expr: Expression) -> ParseResult<(AlbumKind, Album)> {
    fn handle(kind: AlbumKind, mut rand: RandIter) -> ParseResult<(AlbumKind, Album)> {
        let name = get_rand_diary!(rand, Expression::String)?;
        let published_at = get_rand_diary!(rand, Expression::Tuple).and_then(parse_date)?;
        let link_to_diary = match get_rand_diary!(rand, Expression::Tuple) {
            Ok(l) => parse_date(l).map(Some),
            Err(Error::OperandMismatch) => Ok(None),
            Err(err) => Err(err),
        }?;

        Ok((kind, Album::new(name, published_at, link_to_diary)))
    }

    let l = unwrap_expr!(expr, Expression::Tuple).ok_or(Error::IllegalElement)?;
    match_keyword! { l, |rand| {
        "studio" => handle(AlbumKind::StudioAlbum, rand),
        "livealbum" => handle(AlbumKind::LiveAlbum, rand),
        "compilation" => handle(AlbumKind::Compilation, rand),
        "studioandlive" => handle(AlbumKind::StudioAndLive, rand),
        "concert" => handle(AlbumKind::Concert, rand)
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
