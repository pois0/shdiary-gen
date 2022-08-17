use std::{
    collections::BTreeMap,
    env,
    fmt::Debug,
    fs::{self, metadata, DirEntry, File},
    io::{self, BufReader, BufWriter},
    path::{Path, PathBuf},
    string::FromUtf8Error,
};

use index_gen::generate_index;
use post_gen::generate_monthly;
use sexp::ParseError;

mod html;
mod post_gen;
mod sexp;
mod util;
mod index_gen;

#[derive(Debug)]
enum Error {
    IOError(io::Error),
    Utf8Error(FromUtf8Error),
    PathNameError(String),
    ParseError(ParseError),
}

fn main() -> Result<(), Error> {
    let mut current_path = env::current_dir().map_err(Error::IOError)?;
    let cd_dir = fs::read_dir(current_path.clone()).map_err(Error::IOError)?;
    current_path.push("public");
    let public_path = current_path;
    mkdir_if_not_exists(public_path.clone()).map_err(Error::IOError)?;

    let mut years: BTreeMap<u32, Vec<bool>> = BTreeMap::new();

    for year_dir in cd_dir.into_iter().filter_map(|res| res.ok()) {
        let month_path = year_dir.path();
        let metadata = metadata(month_path).map_err(Error::IOError)?;
        if metadata.is_file() {
            continue;
        }
        let month_list = fs::read_dir(year_dir.path()).map_err(Error::IOError)?;
        let mut months = vec![false; 12];

        if year_dir
            .file_name()
            .to_str()
            .map_or(false, |s| s == "public")
        {
            continue;
        }

        let year_num = if let Ok(num) = path_name_to_usize(&year_dir) {
            num
        } else {
            continue
        };
        let year_path = {
            let mut tmp = public_path.clone();
            tmp.push(format!("{}", year_num));
            tmp
        };
        mkdir_if_not_exists(year_path.clone()).map_err(Error::IOError)?;

        for month_dir in month_list.into_iter().filter_map(|res| res.ok()) {
            let day_list = fs::read_dir(month_dir.path()).map_err(Error::IOError)?;
            let mut days = vec![None; 31];

            for day in day_list.into_iter().filter_map(|res| res.ok()) {
                let reader = File::open(day.path())
                    .map(|f| BufReader::new(f))
                    .map_err(Error::IOError)?;

                let post = sexp::parse(reader).map_err(|err| match err {
                    sexp::Error::IOError(err) => Error::IOError(err),
                    sexp::Error::Utf8Error(err) => Error::Utf8Error(err),
                    sexp::Error::ParseError(err) => Error::ParseError(err),
                })?;

                let day_num = path_name_to_usize(&day)?;
                days[day_num - 1] = Some(post);
            }

            let month_num = path_name_to_usize(&month_dir)?;
            months[month_num - 1] = true;

            let file_name = {
                let mut tmp = year_path.clone();
                tmp.push(format!("{:02}.html", month_num));
                tmp
            };
            File::create(file_name)
                .and_then(|f| {
                    let mut buf = BufWriter::new(f);
                    generate_monthly(&mut buf, year_num as i32, month_num as u32, days)
                })
                .map_err(Error::IOError)?;
        }

        years.insert(year_num as u32, months);
    }

    let index_file_name = {
        let mut tmp = public_path.clone();
        tmp.push("index.html");
        tmp
    };
    File::create(index_file_name)
        .and_then(|f| {
            let mut buf = BufWriter::new(f);
            generate_index(&mut buf, years.iter())
        })
        .map_err(Error::IOError)?;

    Ok(())
}

fn mkdir_if_not_exists(path: PathBuf) -> io::Result<()> {
    let exists = path.try_exists()?;
    if !exists {
        fs::create_dir(path)?;
    }

    Ok(())
}

fn path_name_to_usize(entry: &DirEntry) -> Result<usize, Error> {
    Path::new(&entry.file_name())
        .file_stem()
        .ok_or_else(|| path_name_err(&entry))
        .and_then(|s| {
            s.to_str()
                .and_then(|s| s.parse::<usize>().ok())
                .ok_or_else(|| path_name_err(&entry))
        })
}

fn path_name_err(day: &DirEntry) -> Error {
    let path = day
        .path()
        .as_path()
        .to_str()
        .map_or("".to_string(), |s| s.to_string());
    Error::PathNameError(path)
}
