use std::io::{self, Write};

use crate::{albums::AlbumIndex, html::HtmlWriter};

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
            self.writer.start("ul")?;

            for album in artist.albums() {
                self.writer.start("li")?;
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
                self.writer.end("li")?;
            }

            self.writer.end("ul")?;
            self.writer.end("dd")?;
        }

        self.writer.end("dl")?;
        self.writer.end("body")?;
        self.writer.end("html")?;
        Ok(())
    }
}

pub fn generate_albums<W: Write>(writer: &mut W, album_index: AlbumIndex) -> io::Result<()> {
    let mut gen = AlbumsGenerator::new(writer);
    gen.generate(album_index)
}
