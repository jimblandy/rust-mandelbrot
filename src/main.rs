#![allow(dead_code, unused_variables)]

extern crate num;
extern crate image;

use image::ColorType;
use image::png::PNGEncoder;
use num::Complex;
use std::fs::File;
use std::io::Write;
use std::str::FromStr;

type C = f64;

fn count_escape(c: Complex<C>, limit: usize) -> Option<usize> {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 0..limit {
        z = z*z + c;
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
    }

    return None;
}

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
    assert_eq!(parse_pair::<i32>("", ','), None);
    assert_eq!(parse_pair::<i32>("10,20", ','), Some((10, 20)));
    assert_eq!(parse_pair::<i32>("10,20xy", ','), None);
    assert_eq!(parse_pair::<f64>("0.5x", 'x'), None);
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
                  upper_left: (C, C),
                  lower_right: (C, C))
    -> (C, C)
{
    // It might be nicer to find the position of the *middle* of the pixel,
    // instead of its upper left corner, but this is easier to write tests for.
    let (width, height) = (lower_right.0 - upper_left.0,
                           upper_left.1 - lower_right.1);
    (upper_left.0 + pixel.0 as C * width  / bounds.0 as C,
     upper_left.1 - pixel.1 as C * height / bounds.1 as C)
}

#[test]
fn test_pixel_to_point() {
    assert_eq!(pixel_to_point((100, 100), (25, 75),
                              (-1.0, 1.0), (1.0, -1.0)),
               (-0.5, -0.5));
}

fn main() {
    let args : Vec<String> = std::env::args().collect();

    if args.len() != 5 {
        writeln!(std::io::stderr(),
                 r"Usage: mandelbrot FILE PIXELS UPPERLEFT LOWERRIGHT
Example: {} mandel.png 1000x750 -1.20,0.35 -1,0.20
", args[0]).unwrap();
        std::process::exit(1);
    }

    let bounds = parse_pair::<usize>(&args[2], 'x')
        .expect("error parsing image dimensions");
    let upper_left = parse_pair::<C>(&args[3], ',')
        .expect("error parsing upper left corner point");
    let lower_right = parse_pair::<C>(&args[4], ',')
        .expect("error parsing lower right corner point");

    // Now that we've checked the rest of our arguments, try opening the file
    // before we bother doing the computation.
    let output = File::create(&args[1])
        .expect("error opening output file");

    let mut pixels = vec![0; bounds.0 * bounds.1];

    for r in 0 .. bounds.1 {
        for c in 0 .. bounds.0 {
            let point = pixel_to_point(bounds, (c, r),
                                       upper_left, lower_right);
            pixels[r * bounds.0 + c] =
                match count_escape(Complex { re: point.0, im: point.1 }, 255) {
                    None => 0,
                    Some(count) => 255 - count as u8
                };
        }
    }

    let encoder = PNGEncoder::new(output);
    encoder.encode(&pixels[..],
                   bounds.0 as u32, bounds.1 as u32,
                   ColorType::Gray(8))
        .expect("error writing PNG file");
}
