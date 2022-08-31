use crate::util::{calc_hash, push_path};
use image::ImageFormat;
use image::{io::Reader as ImageReader, ImageError};
use log::{info, warn, debug};
use std::fs::{create_dir_all, hard_link, File};
use std::io::{BufReader, BufWriter, ErrorKind, Read, Write};
use std::{io, path::PathBuf};

#[derive(Clone, Debug)]
pub struct ImagePath {
    image_name: String,
}

impl ImagePath {
    pub const fn new(image_name: String) -> ImagePath {
        Self { image_name }
    }

    pub fn thumbnail_path(&self) -> String {
        format!("/img/{}", self.thumbnail_name())
    }

    fn thumbnail_name(&self) -> String {
        format!("{}-thumb.jpeg", self.image_name)
    }

    pub fn actual_path(&self) -> String {
        format!("/img/{}", self.actual_name())
    }

    fn actual_name(&self) -> String {
        format!("{}.webp", self.image_name)
    }

    fn hash_name(&self) -> String {
        format!("{}.xxh3", self.image_name)
    }
}

type ImgResult<T> = Result<T, Error>;

pub enum Error {
    IOError(io::Error),
    ImageError(ImageError),
}

pub struct ImageConverter {
    src_dir: PathBuf,
    dst_dir: PathBuf,
    cache_dir: PathBuf,
}

impl ImageConverter {
    pub fn new(src_dir: PathBuf, dst_dir: PathBuf, cache_dir: PathBuf) -> io::Result<Self> {
        create_dir_all(&src_dir)?;
        create_dir_all(&dst_dir)?;
        create_dir_all(&cache_dir)?;
        Ok(Self {
            src_dir,
            dst_dir,
            cache_dir,
        })
    }

    pub fn convert_image(&self, file_name: String) -> ImgResult<ImagePath> {
        debug!("Converting a image: \"{}\"", file_name);
        let src = push_path(&self.src_dir, &file_name);
        let base_name = file_name
            .split('.')
            .nth(0)
            .unwrap_or(&file_name)
            .to_string();
        let image_path = ImagePath::new(base_name);

        let thumbnail_cache_path = push_path(&self.cache_dir, &file_name);

        let cache_hash_path = push_path(&self.cache_dir, &image_path.hash_name());
        let cache_hash = loop {
            let mut f = match File::open(&cache_hash_path) {
                Ok(f) => f,
                Err(err) => {
                    if err.kind() == ErrorKind::NotFound {
                        break None;
                    } else {
                        return Err(Error::IOError(err));
                    }
                }
            };
            let mut buf = [0u8; 64 / 8];
            f.read(&mut buf).map_err(Error::IOError)?;
            break Some(u64::from_ne_bytes(buf));
        };
        let hash = calc_hash(&src).map_err(Error::IOError)?;

        if cache_hash.filter(|&it| it == hash).is_none() {
            info!("Genrating a thumbnail of \"{}\".", &file_name);
            Self::save_hash(hash, &cache_hash_path)?;
            Self::generate_thumbnail(&src, &thumbnail_cache_path)?;
        }
        Self::link_image(
            &thumbnail_cache_path,
            &push_path(&self.dst_dir, &image_path.thumbnail_name()),
        )
        .or_else(|err| {
            if err.kind() == ErrorKind::NotFound {
                warn!(
                    "The hash file exists, but the thumbnail doesn't exist: \"{}\".",
                    file_name
                );
                Self::generate_thumbnail(&src, &thumbnail_cache_path)?;
                Self::link_image(
                    &thumbnail_cache_path,
                    &push_path(&self.dst_dir, &image_path.thumbnail_name()),
                )
                .map_err(Error::IOError)
            } else {
                Err(Error::IOError(err))
            }
        })?;
        Self::link_image(&src, &push_path(&self.dst_dir, &image_path.actual_name()))
            .map_err(Error::IOError)?;
        Ok(image_path)
    }

    fn save_hash(hash: u64, path: &PathBuf) -> ImgResult<()> {
        let binary = hash.to_ne_bytes();
        let mut writer = File::create(path).map_err(Error::IOError)?;
        writer.write(&binary).map_err(Error::IOError)?;
        Ok(())
    }

    fn link_image(original: &PathBuf, link: &PathBuf) -> io::Result<()> {
        hard_link(original, link)
    }

    fn generate_thumbnail(src: &PathBuf, dst: &PathBuf) -> ImgResult<()> {
        let reader = File::open(src).map_err(Error::IOError)?;
        let img = ImageReader::with_format(BufReader::new(reader), ImageFormat::WebP)
            .decode()
            .map_err(Error::ImageError)?;
        let img = img.thumbnail(300, 96);

        let writer = File::create(dst).map_err(Error::IOError)?;
        img.write_to(&mut BufWriter::new(writer), ImageFormat::Jpeg)
            .map_err(Error::ImageError)?;

        Ok(())
    }
}
