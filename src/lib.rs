use image::{DynamicImage, GrayImage, ImageError, ImageFormat};
use num_traits::cast::ToPrimitive;
use num_traits::NumCast;
use std::io::{BufRead, Seek, Write};

pub const SIGMA_DEFAULT: f32 = 1.8;
pub const SHADOW_DEFAULT: u8 = 150;

pub fn sengaka<R, W>(
  r: R,
  w: &mut W,
  input_ext: &str,
  output_ext: &str,
  sigma: f32,
  shadow: u8,
) -> Result<(), Error>
where
  R: BufRead + Seek,
  W: Write,
{
  let input_format = detect_format(input_ext).ok_or(Error::UnsupportedFormat)?;
  let output_format = detect_format(output_ext).ok_or(Error::UnsupportedFormat)?;
  sengaka_with(r, w, input_format, output_format, sigma, shadow)
}

pub fn sengaka_with<R, W>(
  r: R,
  w: &mut W,
  input_format: ImageFormat,
  output_format: ImageFormat,
  sigma: f32,
  shadow: u8,
) -> Result<(), Error>
where
  R: BufRead + Seek,
  W: Write,
{
  let img = image::load(r, input_format)?;
  let img = process(img, sigma, shadow)?;
  img.write_to(w, output_format)?;
  Ok(())
}

fn process(img: DynamicImage, sigma: f32, shadow: u8) -> Result<DynamicImage, Error> {
  let mut gimg = img.to_luma();
  let mut gimg2 = gimg.clone();
  image::imageops::invert(&mut gimg2);
  let gimg2 = image::imageops::blur(&mut gimg2, sigma);
  color_dodge(&mut gimg, &gimg2);
  levels(&mut gimg, shadow);

  Ok(DynamicImage::ImageRgba8(
    DynamicImage::ImageLuma8(gimg).to_rgba(),
  ))
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
  AlphaNotSupported,
  UnsupportedFormat,
}

impl std::fmt::Display for Error {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match self {
      Error::Image(i) => write!(f, "{}", i),
      Error::AlphaNotSupported => write!(f, "alpha image is not supported"),
      Error::UnsupportedFormat => write!(f, "unsupported format"),
    }
  }
}

impl From<ImageError> for Error {
  fn from(e: ImageError) -> Error {
    Error::Image(e)
  }
}

fn color_dodge(bottom: &mut GrayImage, top: &GrayImage) {
  let max = std::u8::MAX.to_f32().unwrap();
  bottom.pixels_mut().zip(top.pixels()).for_each(|(b, t)| {
    let (bl, tl) = (
      b.data[0].to_f32().unwrap() / max,
      t.data[0].to_f32().unwrap() / max,
    );
    let l = if tl == 1.0 { 1.0 } else { bl / (1.0 - tl) };
    b.data[0] = NumCast::from(if l > 1.0 { 1.0 } else { l } * max).unwrap();
  });
}

fn levels(img: &mut GrayImage, shadow: u8) {
  let max = std::u8::MAX.to_f32().unwrap();
  let s = shadow.to_f32().unwrap() / max;
  let b = 1.0 - s;
  img.pixels_mut().for_each(|p| {
    let l = p.data[0].to_f32().unwrap() / max;
    let nl = if s == 0.0 {
      if l == 1.0 {
        1.0
      } else {
        0.0
      }
    } else if l <= s {
      0.0
    } else {
      (l - s) / b
    };
    p.data[0] = NumCast::from(nl * max).unwrap();
  });
}
