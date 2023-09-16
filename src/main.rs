mod axis;
mod color;
mod depth_image;
mod point;
mod position;
mod rasterize;

use crate::axis::axis::{A, B, G, R, S, T, X, Y, Z};
use image::Rgba;
use std::fs::File;
use std::io::BufRead;
use std::ops::Range;
use std::path::Path;
use std::slice::Iter;
use std::str::FromStr;
use std::{env, io};

use crate::color::Color;
use crate::depth_image::Image;
use crate::point::{Point, Points};
use crate::position::Position;
use crate::rasterize::scanline;

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

fn read_args<P>(fields: Iter<&str>) -> Result<Vec<P>, <P as FromStr>::Err>
where
    P: FromStr,
{
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

fn draw_points(img: &mut Image, points: Points<8>, depth: bool, s_rgb: bool) {
    for point in points {
        if point[X] < 0f64 || point[Y] < 0f64 {
            continue;
        }
        let (x, y) = (point[X] as u32, point[Y] as u32);
        if x < img.width() && y < img.height() {
            let pixel: Rgba<u8> = if s_rgb { point.pixel_s_rgb() } else { point.pixel() };
            if !depth || point[Z] < img.depth(x, y) {
                img.put_pixel(x, y, pixel, if depth { Some(point[Z]) } else { None });
            }
        }
    }
}

fn draw_triangle(mut img: &mut Image, points: &mut Points<8>, depth: bool, s_rgb: bool, hyp: bool, cull: bool) {
    if cull && points.is_back_face() {
        return;
    }

    let mut triangles: Vec<Points<8>> = vec![];
    triangles.push(points.clone());

    for mut triangle in triangles {
        if hyp {
            triangle.divide_by_w(&Box::from([X, Y, Z, R, G, B, A]));
        } else {
            triangle.divide_by_w(&Box::from([X, Y]));
        }

        triangle.transform_to_viewport(img.width(), img.height());

        if hyp {
            triangle.undivide_by_w(&Box::from([Z, R, G, B, A]));
        }

        draw_points(&mut img, scanline(triangle[0], triangle[1], triangle[2]), depth, s_rgb);
    }
}

fn main() {
    if env::args().len() < 2 {
        return;
    }

    let in_filename = env::args().nth(1).unwrap();

    let mut out_filename: String = String::default();
    let mut img: Image = Image::default();

    let mut position_buf: Vec<Position> = vec![];
    let mut color_buf: Vec<Color> = vec![];
    // let mut texcoord_buf = vec![];
    // let mut pointsize_buf = vec![];
    let mut element_buf: Vec<usize> = vec![];

    let mut depth: bool = false;
    let mut s_rgb: bool = false;
    let mut hyp: bool = false;
    // let mut fsaa:u8 = 0;
    let mut cull: bool = false;
    // let mut decals: bool = false;
    // let mut frustum: bool = false;

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

                        img = Image::from_pixel(dim[0], dim[1], Rgba([0, 0, 0, 0]));

                        out_filename = String::from(fields[3]);
                        if let Err(err) = img.save(out_filename.clone()) {
                            eprintln!("{}", err);
                        }
                    }
                    "depth" => {
                        depth = true;
                    }
                    "s_rgb" => {
                        s_rgb = true;
                    }
                    "hyp" => {
                        hyp = true;
                    }
                    "fsaa" => {}
                    "cull" => {
                        cull = true;
                    }
                    "decals" => {}
                    "frustum" => {}
                    "texture" => {}
                    "uniformMatrix" => {}
                    "position" => {
                        if let Ok(args) = read_args::<f64>(fields[1..].iter()) {
                            let size = args[0] as usize;
                            if !(1..=4).contains(&size) {
                                invalid = true;
                                continue;
                            }

                            position_buf.clear();
                            for j in (1..=args.len() - size).step_by(size) {
                                let position = args[j..j + size].to_vec();
                                position_buf.push(Position::new(position));
                            }
                        } else {
                            invalid = true;
                        }
                    }
                    "color" => {
                        if let Ok(args) = read_args::<f64>(fields[1..].iter()) {
                            let size = args[0] as usize;
                            if !(3..=4).contains(&size) {
                                invalid = true;
                                continue;
                            }

                            color_buf.clear();
                            for j in (1..=args.len() - size).step_by(size) {
                                let color = args[j..j + size].to_vec();
                                color_buf.push(Color::new(color));
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
                                let mut points: Points<8> =
                                    merge_data(position_buf.clone(), color_buf.clone(), first + j..first + j + 3);

                                draw_triangle(&mut img, &mut points, depth, s_rgb, hyp, cull);
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

                                draw_triangle(&mut img, &mut points, depth, s_rgb, hyp, cull);
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
