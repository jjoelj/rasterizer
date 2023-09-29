use std::fs::File;
use std::io::BufRead;
use std::path::Path;
use std::slice::Iter;
use std::str::FromStr;
use std::{env, io};

use image::{io::Reader as ImageReader, Rgba, Rgba32FImage};
use ndarray::Array2;

use crate::color::Color;
use crate::depth_image::DepthImage;
use crate::draw::{draw_point, draw_triangle};
use crate::point::{Point, Points};
use crate::position::Position;

mod axis;
mod color;
mod depth_image;
mod draw;
mod point;
mod position;
mod rasterize;

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

fn main() {
    if env::args().len() < 2 {
        return;
    }

    let in_filename = env::args().nth(1).unwrap();

    let mut out_filename: String = String::default();
    let mut img: DepthImage = DepthImage::default();

    let mut texture: Option<Rgba32FImage> = None;

    let mut position_buf: Vec<Position> = vec![];
    let mut color_buf: Vec<Color> = vec![];
    let mut texcoord_buf: Vec<[f64; 2]> = vec![];
    let mut pointsize_buf: Vec<f64> = vec![];
    let mut element_buf: Vec<usize> = vec![];

    let mut depth: bool = false;
    let mut s_rgb: bool = false;
    let mut hyp: bool = false;
    let mut fsaa: u32 = 1;
    let mut cull: bool = false;
    let mut decals: bool = false;
    // let mut frustum: bool = false;

    let mut uniform_matrix: Array2<f64> = Array2::eye(4);

    let mut line_no = 0;
    let mut invalid = false;
    let mut err = "Unknown";

    if let Ok(lines) = read_lines(in_filename) {
        for _line in lines {
            if invalid {
                eprintln!("Error on line {}: {}.", line_no, err);
                invalid = false;
                err = "Unknown";
            }
            line_no += 1;

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
                            err = "Invalid dimensions";
                            continue;
                        }

                        img = DepthImage::from_pixel(dim[0], dim[1], Rgba([0f32; 4]), fsaa);

                        out_filename = String::from(fields[3]);
                        if let Err(err) = img.save(out_filename.clone(), s_rgb) {
                            eprintln!("{}", err);
                        }
                    }
                    "depth" => {
                        depth = true;
                    }
                    "s_rgb" | "sRGB" => {
                        s_rgb = true;
                    }
                    "hyp" => {
                        hyp = true;
                    }
                    "fsaa" => {
                        if let Ok(_fsaa) = fields[1].parse::<u32>() {
                            if !(1..=8).contains(&_fsaa) {
                                invalid = true;
                                err = "Value must be within the range [1, 8]";
                                continue;
                            }

                            fsaa = _fsaa;
                            img = DepthImage::from_pixel(img.width(), img.height(), Rgba([0f32; 4]), fsaa);
                        } else {
                            invalid = true;
                            err = "Value must be an integer";
                        }
                    }
                    "cull" => {
                        cull = true;
                    }
                    "decals" => {
                        decals = true;
                    }
                    "frustum" => {}
                    "texture" => {
                        let texture_filename = String::from(fields[1]);
                        if let Ok(file) = ImageReader::open(texture_filename) {
                            if let Ok(image) = file.decode() {
                                texture = Some(image.into_rgba32f());
                            } else {
                                invalid = true;
                                err = "Unable to decode texture";
                            }
                        } else {
                            invalid = true;
                            err = "Unable to open file";
                        }
                    }
                    "uniformMatrix" => {
                        if let Ok(args) = read_args::<f64>(fields[1..].iter()) {
                            for j in 0..16 {
                                uniform_matrix[[j % 4, j / 4]] = args[j];
                            }
                        }
                    }
                    "position" => {
                        if let Ok(args) = read_args::<f64>(fields[1..].iter()) {
                            let size = args[0] as usize;
                            if !(1..=4).contains(&size) {
                                invalid = true;
                                err = "Invalid size";
                                continue;
                            }

                            position_buf.clear();
                            for j in (1..=args.len() - size).step_by(size) {
                                let position = args[j..j + size].to_vec();
                                position_buf.push(Position::new(position));
                            }
                        } else {
                            invalid = true;
                            err = "Invalid values";
                        }
                    }
                    "color" => {
                        if let Ok(args) = read_args::<f64>(fields[1..].iter()) {
                            let size = args[0] as usize;
                            if !(3..=4).contains(&size) {
                                invalid = true;
                                err = "Invalid size";
                                continue;
                            }

                            color_buf.clear();
                            for j in (1..=args.len() - size).step_by(size) {
                                let color = args[j..j + size].to_vec();
                                color_buf.push(Color::new(color));
                            }
                        } else {
                            invalid = true;
                            err = "Invalid values";
                        }
                    }
                    "texcoord" => {
                        if let Ok(args) = read_args::<f64>(fields[1..].iter()) {
                            let size = args[0] as usize;
                            if size != 2 {
                                invalid = true;
                                err = "Invalid size";
                                continue;
                            }

                            texcoord_buf.clear();
                            for j in (1..=args.len() - 2).step_by(2) {
                                let texcoord: [f64; 2] = [args[j], args[j + 1]];
                                texcoord_buf.push(texcoord);
                            }
                        } else {
                            invalid = true;
                            err = "Invalid values";
                        }
                    }
                    "pointsize" => {
                        if let Ok(args) = read_args::<f64>(fields[1..].iter()) {
                            let size = args[0] as usize;
                            if size != 1 {
                                invalid = true;
                                err = "Invalid size";
                                continue;
                            }

                            pointsize_buf = args[1..].to_vec();
                        } else {
                            invalid = true;
                            err = "Invalid values";
                        }
                    }
                    "elements" => {
                        if let Ok(elements) = read_args::<usize>(fields[1..].iter()) {
                            element_buf = elements;
                        } else {
                            invalid = true;
                            err = "Invalid values";
                        }
                    }
                    "drawArraysTriangles" => {
                        if let Ok(args) = read_args::<usize>(fields[1..].iter()) {
                            let first = args[0];
                            let count = args[1];

                            for j in (0..=count - 3).step_by(3) {
                                let mut points: Points<10> = Points::<10>::from(
                                    position_buf.clone(),
                                    color_buf.clone(),
                                    texcoord_buf.clone(),
                                    first + j..first + j + 3,
                                );

                                draw_triangle(
                                    &mut img,
                                    &mut points,
                                    &uniform_matrix,
                                    &texture,
                                    depth,
                                    hyp,
                                    cull,
                                    decals,
                                );
                            }

                            if let Err(err) = img.save(out_filename.clone(), s_rgb) {
                                eprintln!("{}", err);
                            }
                        } else {
                            invalid = true;
                            err = "Invalid values";
                        }
                    }
                    "drawElementsTriangles" => {
                        if let Ok(args) = read_args::<usize>(fields[1..].iter()) {
                            let count = args[0];
                            let offset = args[1];

                            for j in (0..=count - 3).step_by(3) {
                                let mut temp_position_buf = vec![];
                                let mut temp_color_buf = vec![];
                                let mut temp_texcoord_buf = vec![];

                                for k in 0..3 {
                                    temp_position_buf.push(position_buf[element_buf[offset + j + k]]);
                                    if element_buf[offset + j + k] < color_buf.len() {
                                        temp_color_buf.push(color_buf[element_buf[offset + j + k]]);
                                    }
                                    if element_buf[offset + j + k] < texcoord_buf.len() {
                                        temp_texcoord_buf.push(texcoord_buf[element_buf[offset + j + k]]);
                                    }
                                }

                                let mut points: Points<10> =
                                    Points::<10>::from(temp_position_buf, temp_color_buf, temp_texcoord_buf, 0..3);

                                draw_triangle(
                                    &mut img,
                                    &mut points,
                                    &uniform_matrix,
                                    &texture,
                                    depth,
                                    hyp,
                                    cull,
                                    decals,
                                );
                            }

                            if let Err(err) = img.save(out_filename.clone(), s_rgb) {
                                eprintln!("{}", err);
                            }
                        } else {
                            invalid = true;
                            err = "Invalid values";
                        }
                    }
                    "drawArraysPoints" => {
                        if let Ok(args) = read_args::<usize>(fields[1..].iter()) {
                            let first = args[0];
                            let count = args[1];

                            for j in first..first + count {
                                let mut point: Point<11> = Point::<11>::from((
                                    position_buf[j],
                                    if j < color_buf.len() {
                                        color_buf[j]
                                    } else {
                                        Color::new(vec![0f64; 4])
                                    },
                                    [f64::default(); 2], // No texcoords for points
                                    pointsize_buf[j],
                                ));

                                draw_point(&mut img, &mut point, &uniform_matrix, &texture, depth, decals);
                            }

                            if let Err(err) = img.save(out_filename.clone(), s_rgb) {
                                eprintln!("{}", err);
                            }
                        } else {
                            invalid = true;
                            err = "Invalid values";
                        }
                    }
                    _ => {}
                }
            }
        }
        if invalid {
            eprintln!("Error on line {}: {}.", line_no, err);
        }
    }
}
