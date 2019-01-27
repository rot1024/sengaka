use image::{ImageError, ImageFormat};
use std::io::{BufRead, Seek, Write};

pub const DELTA_DEFAULT: f64 = 1.0;

pub fn sengaka<R, W>(r: R, w: &mut W, input_ext: &str, output_ext: &str) -> Result<(), Error>
where
  R: BufRead + Seek,
  W: Write,
{
  let input_format = detect_format(input_ext).ok_or(Error::UnsupportedFormat)?;
  let output_format = detect_format(output_ext).ok_or(Error::UnsupportedFormat)?;
  let img = image::load(r, input_format)?;

  // process

  img.write_to(w, output_format)?;
  Ok(())
}

pub fn detect_format(ext: &str) -> Option<ImageFormat> {
  Some(match ext {
    "jpg" | "jpeg" => image::ImageFormat::JPEG,
    "png" => image::ImageFormat::PNG,
    "gif" => image::ImageFormat::GIF,
    "webp" => image::ImageFormat::WEBP,
    "tif" | "tiff" => image::ImageFormat::TIFF,
    "tga" => image::ImageFormat::TGA,
    "bmp" => image::ImageFormat::BMP,
    "ico" => image::ImageFormat::ICO,
    "hdr" => image::ImageFormat::HDR,
    "pbm" | "pam" | "ppm" | "pgm" => image::ImageFormat::PNM,
    _ => return None,
  })
}

#[derive(Debug)]
pub enum Error {
  Image(ImageError),
  UnsupportedFormat,
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Error::Image(i) => write!(f, "{}", i),
      Error::UnsupportedFormat => write!(f, "unsupported format"),
    }
  }
}

impl From<ImageError> for Error {
  fn from(e: ImageError) -> Error {
    Error::Image(e)
  }
}
