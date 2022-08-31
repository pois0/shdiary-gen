use std::{
    fs::File,
    io::{self, Read},
    path::PathBuf,
};

use chrono::{Datelike, Weekday};
use xxhash_rust::xxh3::Xxh3;

pub fn weekday_ja<D: Datelike>(date: &D) -> &str {
    match date.weekday() {
        Weekday::Mon => "月",
        Weekday::Tue => "火",
        Weekday::Wed => "水",
        Weekday::Thu => "木",
        Weekday::Fri => "金",
        Weekday::Sat => "土",
        Weekday::Sun => "日",
    }
}

pub fn push_path(origin: &PathBuf, elem: &str) -> PathBuf {
    let mut tmp = origin.clone();
    tmp.push(elem);
    tmp
}

const BUFFER_SIZE: usize = 8192;

pub fn calc_hash(src: &PathBuf) -> io::Result<u64> {
    let mut reader = File::open(src)?;
    let mut buf = Vec::with_capacity(BUFFER_SIZE);
    unsafe {
        buf.set_len(BUFFER_SIZE);
    }
    let mut hash_ctx = Box::new(Xxh3::new());
    loop {
        let size = reader.read(&mut buf)?;
        hash_ctx.update(&buf[..size]);
        if size != BUFFER_SIZE {
            break Ok(hash_ctx.digest());
        }
    }
}
