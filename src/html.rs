use std::io::{Write, self};

pub struct HtmlWriter<'a, W: Write> {
    writer: &'a mut W,
}

impl<'a, W: Write> HtmlWriter<'a, W> {
    pub fn new(writer: &'a mut W) -> Self {
        Self { writer }
    }

    pub fn start<'slf, 'str>(&'slf mut self, name: &'str str) -> io::Result<()> {
        write!(self.writer, "<{}>", name)
    }

    pub fn start_attr<'slf, 'str>(
        &'slf mut self,
        name: &'str str,
        attr: &[(&'str str, &'str str)],
    ) -> io::Result<()> {
        let attributes = attr
            .iter()
            .map(|(k, v)| format!(r#"{}="{}""#, k, v))
            .reduce(|mut acc, e| {
                acc.push_str(&e);
                acc
            })
            .map_or("".to_string(), |s| s);
        write!(self.writer, "<{} {}>", name, &attributes)
    }

    pub fn end<'slf, 'str>(&'slf mut self, name: &'str str) -> io::Result<()> {
        write!(self.writer, "</{}>", name)
    }

    pub fn doctype(&mut self) -> io::Result<()> {
        write!(self.writer, "<!DOCTYPE html>")
    }
}

impl <'a, W: Write> Write for HtmlWriter<'a, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}
