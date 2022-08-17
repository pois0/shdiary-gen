use std::io::{self, Write};

use crate::html::HtmlWriter;

struct IndexGenerator<'a, W: Write> {
    writer: HtmlWriter<'a, W>,
}

impl<'a, W: Write> IndexGenerator<'a, W> {
    fn new(writer: &'a mut W) -> Self {
        Self {
            writer: HtmlWriter::new(writer),
        }
    }

    fn generate<'t, T: Iterator<Item = (&'t u32, &'t Vec<bool>)>>(
        &'a mut self,
        list: T,
    ) -> io::Result<()>
    where
        't: 'a,
    {
        self.writer.doctype()?;
        self.writer.start_attr("html", &[("lang", "ja")])?;
        self.writer.start("head")?;
        self.writer.start_attr("meta", &[("charset", "utf-8")])?;
        self.writer.start("title")?;
        write!(self.writer, "{}", "Natuka.ge")?;
        self.writer.end("title")?;
        self.writer.end("head")?;
        self.writer.start("body")?;
        self.writer.start("h1")?;
        write!(self.writer, "{}", "Natuka.ge")?;
        self.writer.end("h1")?;
        self.writer.start("hr")?;
        self.writer.start("ul")?;
        for (year, months) in list {
            self.write_year(*year, months)?;
        }
        self.writer.end("ul")?;
        self.writer.end("body")?;
        self.writer.end("html")?;
        Ok(())
    }

    fn write_year(&mut self, year: u32, months: &Vec<bool>) -> io::Result<()> {
        self.writer.start("li")?;
        write!(self.writer, "{}年", year)?;
        for month in months
            .iter()
            .enumerate()
            .filter_map(|(month, &b)| Some(month + 1).filter(|_| b))
            .rev()
        {
            self.writer.start("ul")?;
            self.writer.start("li")?;
            self.writer.start_attr("a", &[("href", &format!("/{}/{:02}", year, month))])?;
            write!(self.writer, "{}月", month)?;
            self.writer.end("li")?;
            self.writer.end("ul")?;
        }
        self.writer.end("li")?;
        Ok(())
    }
}

pub fn generate_index<'a, W: Write, T: Iterator<Item = (&'a u32, &'a Vec<bool>)>>(
    writer: &'a mut W,
    list: T,
) -> io::Result<()> {
    let mut gen = IndexGenerator::new(writer);
    gen.generate(list)?;
    Ok(())
}
