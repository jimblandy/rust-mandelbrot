#![allow(dead_code, unused_variables)]

extern crate bench;

use std::str::FromStr;

/// Parse the string `s` as a coordinate pair, like `"400x600"` or `"1.0,0.5"`.
///
/// Specifically, `s` should have the form <left><sep><right>, where <sep> is
/// the character given by the `separator` argument, and <left> and <right> are both
/// strings that can be parsed by `T::from_str`.
///
/// If `s` has the proper form, return `Some<(x, y)>`. If it doesn't parse
/// correctly, return `None`.
fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(index) => {
            match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
                (Ok(l), Ok(r)) => Some((l, r)),
                _ => None
            }
        }
    }
}

#[test]
fn test_parse_pair() {
    assert_eq!(parse_pair::<i32>("",        ','), None);
    assert_eq!(parse_pair::<i32>("10,",     ','), None);
    assert_eq!(parse_pair::<i32>(",10",     ','), None);
    assert_eq!(parse_pair::<i32>("10,20",   ','), Some((10, 20)));
    assert_eq!(parse_pair::<i32>("10,20xy", ','), None);
    assert_eq!(parse_pair::<f64>("0.5x",    'x'), None);
    assert_eq!(parse_pair::<f64>("0.5x1.5", 'x'), Some((0.5, 1.5)));
}

/// Return the point on the complex plane corresponding to a given pixel in the
/// bitmap.
///
/// `bounds` is a pair giving the width and height of the bitmap. `pixel` is a
/// pair indicating a particular pixel in that bitmap. The `upper_left` and
/// `lower_right` parameters are points on the complex plane designating the
/// area our bitmap covers.
fn pixel_to_point(bounds: (usize, usize),
                  pixel: (usize, usize),
                  upper_left: (f64, f64),
                  lower_right: (f64, f64))
    -> (f64, f64)
{
    // It might be nicer to find the position of the *middle* of the pixel,
    // instead of its upper left corner, but this is easier to write tests for.
    let (width, height) = (lower_right.0 - upper_left.0,
                           upper_left.1 - lower_right.1);
    (upper_left.0 + pixel.0 as f64 * width  / bounds.0 as f64,
     upper_left.1 - pixel.1 as f64 * height / bounds.1 as f64)
}

#[test]
fn test_pixel_to_point() {
    assert_eq!(pixel_to_point((100, 100), (25, 75),
                              (-1.0, 1.0), (1.0, -1.0)),
               (-0.5, -0.5));
}

extern crate num;
use num::Complex;

/// 
fn escapes(mut z: Complex<f64>, c: Complex<f64>, limit: u32) -> Option<u32> {
    for i in 0..limit {
        z = z*z + c;
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
    }

    return None;
}

/// Render a rectangle of the Mandelbrot set into a buffer of pixels.
///
/// The `bounds` argument gives the width and height of the buffer `pixels`,
/// which holds one grayscale pixel per byte. The `upper_left` and `lower_right`
/// arguments specify points on the complex plane corresponding to the upper
/// left and lower right corners of the pixel buffer.
fn render(param: Complex<f64>,
          pixels: &mut [u8], bounds: (usize, usize),
          upper_left: (f64, f64), lower_right: (f64, f64))
{
    assert!(pixels.len() == bounds.0 * bounds.1);

    for r in 0 .. bounds.1 {
        for c in 0 .. bounds.0 {
            let point = pixel_to_point(bounds, (c, r),
                                       upper_left, lower_right);
            pixels[r * bounds.0 + c] =
                match escapes(Complex { re: point.0, im: point.1 }, param, 255) {
                    None => 0,
                    Some(count) => 255 - count as u8
                };
        }
    }
}

extern crate image;

use std::fs::File;
use std::io::Result;
use image::png::PNGEncoder;
use image::ColorType;

/// Write the buffer `pixels`, whose dimensions are given by `bounds`, to the
/// file named `filename`.
fn write_bitmap(filename: &str, pixels: &[u8], bounds: (usize, usize))
    -> Result<()>
{
    let output = try!(File::create(filename));

    let encoder = PNGEncoder::new(output);
    try!(encoder.encode(&pixels[..],
                        bounds.0 as u32, bounds.1 as u32,
                        ColorType::Gray(8)));

    Ok(())
}

extern crate crossbeam;
extern crate atomic_chunks_mut;

use atomic_chunks_mut::AtomicChunksMut;

use std::io::Write;

fn main() {
    let args : Vec<String> = std::env::args().collect();

    if args.len() != 6 {
        writeln!(std::io::stderr(),
                 "Usage: mandelbrot FILE PIXELS UPPERLEFT LOWERRIGHT C")
            .unwrap();
        writeln!(std::io::stderr(),
                 "Example: {} mandel.png 1000x750 -1.20,0.35 -1,0.20 -0.727,0.189",
                 args[0])
            .unwrap();
        std::process::exit(1);
    }

    let bounds = parse_pair(&args[2], 'x')
        .expect("error parsing image dimensions");
    let upper_left = parse_pair(&args[3], ',')
        .expect("error parsing upper left corner point");
    let lower_right = parse_pair(&args[4], ',')
        .expect("error parsing lower right corner point");
    let c = parse_pair(&args[5], ',')
        .expect("error parsing parameter c");

    let mut pixels = vec![0; bounds.0 * bounds.1];
    let area = bounds.0 as f64 * bounds.1 as f64;

    {
        let bands = AtomicChunksMut::new(&mut pixels, bounds.0);
        crossbeam::scope(|scope| {
            for i in 0..8 {
                scope.spawn(|| {
                    let mut count = 0;
                    for (i, band) in &bands {
                        count += 1;
                        let top = i;
                        let height = band.len() / bounds.0;
                        let band_bounds = (bounds.0, height);
                        let band_upper_left = pixel_to_point(bounds, (0, top),
                                                             upper_left, lower_right);
                        let band_lower_right = pixel_to_point(bounds, (bounds.0, top + height),
                                                              upper_left, lower_right);
                        render(Complex { re: c.0, im: c.1 },
                               band, band_bounds, band_upper_left, band_lower_right);
                    }
                });
            }
        });
    }

    write_bitmap(&args[1], &pixels[..], bounds).expect("error writing PNG file");
}
