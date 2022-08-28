use std::io::{self, Bytes, Read};

pub struct StringReader<R: Read> {
    bytes: Bytes<R>,
    chr: Option<u8>,
}

impl<R: Read> StringReader<R> {
    pub fn new(raw_read: R) -> io::Result<Option<Self>> {
        let mut bytes = raw_read.bytes();
        bytes.next().map_or(Ok(None), |chr| {
            let chr = chr?;
            Ok(Some(Self {
                bytes,
                chr: Some(chr),
            }))
        })
    }

    pub const fn chr(&self) -> Option<u8> {
        self.chr
    }

    pub fn seek(&mut self) -> io::Result<()> {
        self.chr = match self.bytes.next() {
            Some(res) => {
                let chr = res?;
                Some(chr)
            }
            None => None,
        };
        Ok(())
    }
}
