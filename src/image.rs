use image::ImageFormat;
use image::{io::Reader as ImageReader, ImageError};
use std::fs::File;
use std::io::{BufReader, BufWriter};
use std::{
    io::{self, Read},
    path::PathBuf,
};

use crate::util::push_path;

#[derive(Clone, Debug)]
pub struct ImagePath {
    image_name: String,
}

impl ImagePath {
    pub const fn new(image_name: String) -> ImagePath {
        Self { image_name }
    }

    fn thumbnail_path(&self) -> String {
        format!("/img/{}-thumb.avif", self.image_name)
    }

    fn actual_path(&self) -> String {
        format!("/img/{}.avif", self.image_name)
    }
}

type ImgResult<T> = Result<T, Error>;

pub enum Error {
    IOError(io::Error),
    ImageError(ImageError),
    FilePathError,
}

fn convert_image(src: PathBuf, dst_dir: &PathBuf) -> ImgResult<ImagePath> {
    let file_name = src
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| name.to_string())
        .ok_or(Error::FilePathError)?;
    let dst = push_path(dst_dir, &file_name);

    let is_cache_exist = check_cache(&file_name)?;
    if !is_cache_exist {
        generate_thumbnail(src, &dst)?;
    }

    Ok(ImagePath::new(file_name))
}

fn check_cache(file_name: &str) -> ImgResult<bool> {
    Ok(false)
}

fn generate_thumbnail(src: PathBuf, dst: &PathBuf) -> ImgResult<()> {
    let reader = File::open(src).map_err(Error::IOError)?;
    let img = ImageReader::with_format(BufReader::new(reader), ImageFormat::Avif)
        .decode()
        .map_err(Error::ImageError)?;
    let img = img.thumbnail(300, 96);

    let writer = File::create(dst).map_err(Error::IOError)?;
    img.write_to(&mut BufWriter::new(writer), ImageFormat::Avif).map_err(Error::ImageError)?;

    Ok(())
}
