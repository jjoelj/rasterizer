mod color;
mod rasterize;
mod point;
mod position;
mod axis;

use image::{Rgba, RgbaImage};
use std::{env, io};
use std::fs::{File};
use std::io::{BufRead};
use std::ops::Range;
use std::path::Path;
use std::slice::Iter;
use std::str::FromStr;
use crate::axis::axis::{X, Y, W, R, G, B, A, S, T};

use crate::color::Color;
use crate::point::{Point, Points};
use crate::position::Position;
use crate::rasterize::scanline;

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>> where P: AsRef<Path>, {
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn read_args<P>(fields: Iter<&str>) -> Result<Vec<P>, <P as FromStr>::Err> where P: FromStr {
    let mut result: Vec<P> = vec![];
    for field in fields {
        let res = field.parse::<P>();

        if res.is_err() {
            return Err(res.err().unwrap());
        }

        result.push(res.ok().unwrap());
    }

    return Ok(result);
}

fn merge_data(position_buf: Vec<Position>, color_buf: Vec<Color>, range: Range<usize>) -> Points<8> {
    let mut result: Vec<Point<8>> = vec![];
    for j in range {
        result.push(Point::from((position_buf[j], color_buf[j])));
    }
    return Points::<8>(result);
}

fn draw_points(img: &mut RgbaImage, points: Points<8>, s_rgb: bool) {
    for point in points {
        if point[X] < 0f64 || point[Y] < 0f64 {
            continue;
        }
        let (x, y) = (point[X] as u32, point[Y] as u32);
        if x < img.width() && y < img.height() {
            let pixel: Rgba<u8> = if s_rgb { point.pixel_s_rgb() } else { point.pixel() };
            img.put_pixel(x, y, pixel);
        }
    }
}

fn main() {
    if env::args().len() < 2 {
        return;
    }

    let in_filename = env::args().nth(1).unwrap();

    let mut out_filename: String = String::default();
    let mut img: RgbaImage = RgbaImage::default();

    let mut position_buf: Vec<Position> = vec![];
    let mut color_buf: Vec<Color> = vec![];
    let mut element_buf: Vec<usize> = vec![];

    let mut s_rgb: bool = false;

    let mut line_no = 0;
    let mut invalid = false;

    if let Ok(lines) = read_lines(in_filename) {
        for (i, _line) in lines.enumerate() {
            line_no += 1;
            if invalid {
                eprintln!("Syntax error on line {}.", i - 1);
                invalid = false;
            }

            if let Ok(line) = _line {
                let trim_line = line.trim();
                let fields = trim_line.split_whitespace().collect::<Vec<&str>>();
                if fields.len() == 0 {
                    continue;
                }

                match fields[0] {
                    "png" => {
                        let mut dim: Vec<u32> = vec![];
                        for field in fields[1..=2].iter() {
                            let parsed = field.parse::<u32>().ok();
                            invalid = parsed == None;

                            if !invalid {
                                dim.push(parsed.unwrap());
                            }
                        }
                        if invalid || dim.len() < 2 {
                            continue;
                        }

                        img = RgbaImage::from_pixel(dim[0], dim[1], Rgba([0, 0, 0, 0]));

                        out_filename = String::from(fields[3]);
                        if let Err(err) = img.save(out_filename.clone()) {
                            eprintln!("{}", err);
                        }
                    }
                    "depth" => {}
                    "s_rgb" => {
                        s_rgb = true;
                    }
                    "hyp" => {}
                    "fsaa" => {}
                    "cull" => {}
                    "decals" => {}
                    "frustum" => {}
                    "texture" => {}
                    "uniformMatrix" => {}
                    "position" => {
                        if let Ok(size) = fields[1].parse::<usize>() {
                            if !(1..=4).contains(&size) {
                                invalid = true;
                                continue;
                            }

                            position_buf.clear();
                            for j in (2..=fields.len() - size).step_by(size) {
                                if let Ok(position) = read_args(fields[j..j + size].iter()) {
                                    position_buf.push(Position::new(position));
                                } else {
                                    position_buf.clear();
                                    invalid = true;
                                    break;
                                }
                            }
                        } else {
                            invalid = true;
                        }
                    }
                    "color" => {
                        if let Ok(size) = fields[1].parse::<usize>() {
                            if !(3..=4).contains(&size) {
                                invalid = true;
                                continue;
                            }

                            color_buf.clear();
                            for j in (2..=fields.len() - size).step_by(size) {
                                if let Ok(color) = read_args(fields[j..j + size].iter()) {
                                    color_buf.push(Color::new(color));
                                } else {
                                    color_buf.clear();
                                    invalid = true;
                                    break;
                                }
                            }
                        } else {
                            invalid = true;
                        }
                    }
                    "texcoord" => {}
                    "pointsize" => {}
                    "elements" => {
                        if let Ok(elements) = read_args::<usize>(fields[1..].iter()) {
                            element_buf = elements;
                        }
                    }
                    "drawArraysTriangles" => {
                        if let Ok(args) = read_args::<usize>(fields[1..].iter()) {
                            let first = args[0];
                            let count = args[1];

                            for j in (0..=count - 3).step_by(3) {
                                let mut points: Points<8> = merge_data(position_buf.clone(), color_buf.clone(), first + j..first + j + 3);
                                points.divide_by_w(W);
                                points.transform_to_viewport(X, Y, img.width(), img.height());
                                points.undivide_by_w(W, &Box::from([R, G, B, A]));

                                let triangle = scanline(points[0], points[1], points[2]);
                                draw_points(&mut img, triangle, s_rgb);
                            }

                            if let Err(err) = img.save(out_filename.clone()) {
                                eprintln!("{}", err);
                            }
                        } else {
                            invalid = true;
                        }
                    }
                    "drawElementsTriangles" => {
                        if let Ok(args) = read_args::<usize>(fields[1..].iter()) {
                            let count = args[0];
                            let offset = args[1];

                            for j in (0..=count - 3).step_by(3) {
                                let mut temp_position_buf = vec![];
                                let mut temp_color_buf = vec![];

                                for k in 0..3 {
                                    temp_position_buf.push(position_buf[element_buf[offset + j + k]]);
                                    if element_buf[offset + j + k] < color_buf.len() {
                                        temp_color_buf.push(color_buf[element_buf[offset + j + k]]);
                                    }
                                }

                                let mut points: Points<8> = merge_data(temp_position_buf, temp_color_buf, 0..3);
                                points.divide_by_w(W);
                                points.transform_to_viewport(X, Y, img.width(), img.height());
                                points.undivide_by_w(W, &Box::from([R, G, B, A]));

                                let triangle = scanline(points[0], points[1], points[2]);
                                draw_points(&mut img, triangle, s_rgb);
                            }

                            if let Err(err) = img.save(out_filename.clone()) {
                                eprintln!("{}", err);
                            }
                        } else {
                            invalid = true;
                        }
                    }
                    "drawArraysPoints" => {}
                    _ => {}
                }
            }
        }
        if invalid {
            eprintln!("Error on line {}.", line_no);
        }
    }
}
