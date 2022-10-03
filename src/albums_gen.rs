use std::io::{self, Write};

use crate::{
    albums::{Album, AlbumIndex},
    html::HtmlWriter,
};

struct AlbumsGenerator<'a, W: Write> {
    writer: HtmlWriter<'a, W>,
}

impl<'a, W: Write> AlbumsGenerator<'a, W> {
    fn new(writer: &'a mut W) -> Self {
        Self {
            writer: HtmlWriter::new(writer),
        }
    }

    fn generate(&mut self, AlbumIndex(artists): AlbumIndex) -> io::Result<()> {
        let title = "Natuka.ge - Albums";
        self.writer.doctype()?;
        self.writer.start_attr("html", &[("lang", "ja")])?;
        self.writer.start("head")?;
        self.writer.start_attr("meta", &[("charset", "utf-8")])?;
        self.writer.start("title")?;
        write!(self.writer, "{}", title)?;
        self.writer.end("title")?;
        self.writer.end("head")?;
        self.writer.start("body")?;
        self.writer.start("h1")?;
        write!(self.writer, "{}", title)?;
        self.writer.end("h1")?;
        self.writer.start_attr("a", &[("href", "/")])?;
        write!(self.writer, "ホーム")?;
        self.writer.end("a")?;
        write!(self.writer, "へ")?;
        self.writer.start("hr")?;
        self.writer.start("dl")?;

        for artist in artists {
            self.writer.start("dt")?;
            self.writer.start("h2")?;
            write!(self.writer, "{}", artist.name())?;
            self.writer.end("h2")?;
            self.writer.end("dt")?;
            self.writer.start("dd")?;

            let album_list = artist.albums();
            self.generate_albums("Studio albums", album_list.studio_album())?;
            self.generate_albums("Live albums", album_list.live_album())?;
            self.generate_albums("Studio and live album", album_list.studio_and_live())?;
            self.generate_albums("Compilations", album_list.compilation())?;
            self.generate_albums("Concerts", album_list.live())?;

            self.writer.end("dd")?;
        }

        self.writer.end("dl")?;
        self.writer.end("body")?;
        self.writer.end("html")?;
        Ok(())
    }

    fn generate_albums(&mut self, title: &str, albums: &[Album]) -> io::Result<()> {
        if albums.is_empty() {
            return Ok(())
        }
        self.writer.start("h3")?;
        write!(self.writer, "{}", title)?;
        self.writer.end("h3")?;
        self.writer.start("ul")?;
        for sa in albums {
            self.writer.start("li")?;
            self.generate_album(sa)?;
            self.writer.end("li")?;
        }
        self.writer.end("ul")?;

        Ok(())
    }

    fn generate_album(&mut self, album: &Album) -> io::Result<()> {
        write!(self.writer, "{}", album.name())?;
        if let Some(diary) = album.link_to_diary() {
            write!(self.writer, " (")?;
            self.writer.start_attr(
                "a",
                &[(
                    "href",
                    &format!("/{}/{:02}#{:02}", diary.year(), diary.month(), diary.day()),
                )],
            )?;
            write!(
                self.writer,
                "{}/{:02}/{:02}",
                diary.year(),
                diary.month(),
                diary.day()
            )?;
            self.writer.end("a")?;
            write!(self.writer, ")")?;
        }

        Ok(())
    }
}

pub fn generate_albums<W: Write>(writer: &mut W, album_index: AlbumIndex) -> io::Result<()> {
    let mut gen = AlbumsGenerator::new(writer);
    gen.generate(album_index)
}
