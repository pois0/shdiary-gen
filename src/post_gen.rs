use std::io::{self, Write};

use crate::date::Date;
use crate::diary_content::{Document, ImageItem, Images, Item, Text, TextItem};
use crate::html::HtmlWriter;
use crate::image::ImagePath;

pub type OutputDocument = Document<ImagePath>;

pub type OutputItem = Item<ImagePath>;

struct PostGenerator<'a, W: Write> {
    writer: HtmlWriter<'a, W>,
}

impl<'a, W: Write> PostGenerator<'a, W> {
    fn new(writer: &'a mut W) -> Self {
        Self {
            writer: HtmlWriter::new(writer),
        }
    }

    fn generate_monthly(
        &mut self,
        year: u32,
        month: u32,
        docs: Vec<Option<OutputDocument>>,
    ) -> io::Result<()> {
        let title = format!("Natuka.ge - {:4}/{:02}", year, month);
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

        for (day, doc) in docs
            .into_iter()
            .enumerate()
            .filter_map(|(day, doc)| doc.map(|doc| (day, doc)))
            .rev()
        {
            let date = Date::new(year, month, (day + 1) as u32)
                .unwrap_or_else(|| panic!("Wrong date: ({}, {}, {})", year, month, day + 1));
            self.generate_daily(&date, &doc)?;
        }

        self.writer.end("dl")?;
        self.writer.end("body")?;
        self.writer.end("html")?;
        Ok(())
    }

    fn generate_daily(&mut self, date: &Date, doc: &OutputDocument) -> io::Result<()> {
        self.writer.start("dt")?;
        self.write_date(date)?;
        self.writer.end("dt")?;

        self.writer.start("dd")?;
        for item in doc.contents() {
            match item {
                Item::Text(txt) => self.write_paragraph(txt),
                Item::List(li) => self.write_list(li),
                Item::Header(txt) => self.write_header(txt),
                Item::Images(images) => self.write_images(images),
            }?;
        }
        self.writer.end("dd")?;

        Ok(())
    }

    fn write_date(&mut self, date: &Date) -> io::Result<()> {
        let id = format!("{}", date.day());
        self.writer.start_attr("h2", &[("id", &id)])?;
        self.writer
            .start_attr("a", &[("href", &format!("#{}", id))])?;
        write!(
            self.writer,
            "{} ({})",
            format!("{}/{:02}/{:02}", date.year(), date.month(), date.day()),
            date.weekday_ja()
        )?;
        self.writer.end("a")?;
        self.writer.end("h2")
    }

    fn write_header(&mut self, txt: &str) -> io::Result<()> {
        self.writer.start("h3")?;
        write!(self.writer, "{}", txt)?;
        self.writer.end("h3")
    }

    fn write_paragraph(&mut self, txt: &Text) -> io::Result<()> {
        self.writer.start("p")?;
        self.write_text(txt)?;
        self.writer.end("p")
    }

    fn write_list(&mut self, items: &Vec<OutputItem>) -> io::Result<()> {
        self.writer.start("ul")?;
        for item in items {
            match item {
                Item::Text(txt) => {
                    self.writer.start("li")?;
                    self.write_text(&txt)?;
                    self.writer.end("li")
                }
                Item::List(li) => self.write_list(&li),
                Item::Header(_) => unreachable!(),
                Item::Images(images) => {
                    self.writer.start("li")?;
                    self.write_images(images)?;
                    self.writer.end("li")
                }
            }?;
        }
        self.writer.end("ul")
    }

    fn write_text(&mut self, txt: &Text) -> io::Result<()> {
        for e in txt {
            match e {
                TextItem::Bold(txt) => {
                    self.writer.start("b")?;
                    write!(self.writer, "{}", txt)?;
                    self.writer.end("b")?;
                }
                TextItem::RawString(txt) => {
                    write!(self.writer, "{}", txt)?;
                }
                TextItem::WebLink(link) => {
                    self.writer.start_attr("a", &[("href", &link.href)])?;
                    write!(self.writer, "{}", link.title)?;
                    self.writer.end("a")?;
                }
                TextItem::PostLink((year, month, day)) => {
                    let href = format!("/{:04}/{:02}#{:02}", year, month, day);
                    write!(self.writer, "(ref. ")?;
                    self.writer.start_attr("a", &[("href", &href)])?;
                    write!(self.writer, "{:04}/{:02}/{:02}", year, month, day)?;
                    self.writer.end("a")?;
                    write!(self.writer, ")")?;
                }
                TextItem::Code(txt) => {
                    self.writer.start("code")?;
                    write!(self.writer, "{}", txt)?;
                    self.writer.end("code")?;
                }
            }
        }
        Ok(())
    }

    fn write_images(&mut self, images: &Images<ImagePath>) -> io::Result<()> {
        write!(self.writer, "{}", images.title)?;
        self.writer.start("table")?;
        self.writer.start("tbody")?;
        self.writer.start("tr")?;
        for ImageItem { data, .. } in &images.items {
            self.writer.start("td")?;
            self.writer
                .start_attr("a", &[("href", &data.actual_path())])?;
            self.writer.start_attr(
                "img",
                &[
                    ("src", &data.thumbnail_path()),
                    ("width", &data.width().to_string()),
                    ("height", &data.height().to_string()),
                ],
            )?;
            self.writer.end("a")?;
            self.writer.end("td")?;
        }
        self.writer.end("tr")?;
        self.writer.start("tr")?;
        for image in &images.items {
            self.writer.start("td")?;
            if let Some(caption) = &image.caption {
                write!(self.writer, "{}", caption)?;
            }
            self.writer.end("td")?;
        }
        self.writer.end("tr")?;
        self.writer.end("tbody")?;
        self.writer.end("table")?;

        Ok(())
    }
}

pub fn generate_monthly<W: Write>(
    writer: &mut W,
    year: u32,
    month: u32,
    docs: Vec<Option<OutputDocument>>,
) -> io::Result<()> {
    let mut gen = PostGenerator::new(writer);
    gen.generate_monthly(year, month, docs)
}
