use image::{Rgba, Rgba32FImage};
use ndarray::Array2;
use palette::Srgb;

use crate::axis::axis::{A, B, G, R, S, T, X, Y, Z};
use crate::depth_image::DepthImage;
use crate::point::{Point, Points};
use crate::rasterize::{square, triangle};

fn overlay_pixels(cur_pixel: Rgba<f32>, pixel: Rgba<f32>) -> [f32; 4] {
    let [r_s, g_s, b_s, a_s] = pixel.0;
    let [r_d, g_d, b_d, a_d] = cur_pixel.0;

    let a = a_s + a_d * (1f32 - a_s);
    let r = a_s / a * r_s + (1f32 - a_s) * a_d / a * r_d;
    let g = a_s / a * g_s + (1f32 - a_s) * a_d / a * g_d;
    let b = a_s / a * b_s + (1f32 - a_s) * a_d / a * b_d;

    return [r, g, b, a];
}

struct Draw<const DIM: usize>();

impl<const DIM: usize> Draw<DIM> {
    fn draw_points(
        img: &mut DepthImage,
        points: Points<DIM>,
        texture: &Option<Rgba32FImage>,
        depth: bool,
        decals: bool,
    ) {
        for point in points {
            if point[X] < 0f64 || point[Y] < 0f64 {
                continue;
            }
            let (x, y) = (point[X] as u32, point[Y] as u32);
            if x < img.width() && y < img.height() {
                let mut pixel: Rgba<f32> = point.pixel();
                if !depth || point[Z] < img.depth(x, y) {
                    let cur_pixel = img.get_pixel(x, y).clone();

                    [pixel[0], pixel[1], pixel[2], pixel[3]] = overlay_pixels(cur_pixel, pixel);

                    if let Some(texture) = texture {
                        let x = (point[S] * texture.width() as f64).rem_euclid(texture.width() as f64) as u32;
                        let y = (point[T] * texture.height() as f64).rem_euclid(texture.height() as f64) as u32;
                        let _temp = texture.get_pixel(x, y).clone();
                        let mut temp: Rgba<f32> = Rgba([0f32; 4]);
                        let texel = Srgb::from_components((_temp[0], _temp[1], _temp[2])).into_linear::<f32>();
                        temp[0] = texel.red;
                        temp[1] = texel.green;
                        temp[2] = texel.blue;
                        temp[3] = _temp[3];
                        if decals {
                            [pixel[0], pixel[1], pixel[2], pixel[3]] = overlay_pixels(pixel, temp);
                        } else {
                            pixel = temp;
                        }
                    }

                    img.put_pixel(x, y, pixel, if depth { Some(point[Z]) } else { None });
                }
            }
        }
    }
}

pub(crate) fn draw_triangle(
    mut img: &mut DepthImage,
    points: &mut Points<10>,
    uniform_matrix: &Array2<f64>,
    texture: &Option<Rgba32FImage>,
    depth: bool,
    hyp: bool,
    cull: bool,
    decals: bool,
) {
    points.multiply_by_matrix(&uniform_matrix);

    if cull && points.is_back_face() {
        return;
    }

    if hyp {
        points.divide_by_w(&Box::from([X, Y, Z, R, G, B, A, S, T]));
    } else {
        points.divide_by_w(&Box::from([X, Y]));
    }

    points.transform_to_viewport(img.width(), img.height());

    let mut triangle = triangle(points[0], points[1], points[2]);

    if hyp {
        triangle.undivide_by_w(&Box::from([Z, R, G, B, A, S, T]));
    }

    Draw::<10>::draw_points(&mut img, triangle, texture, depth, decals);
}

pub(crate) fn draw_point(
    mut img: &mut DepthImage,
    point: &mut Point<11>,
    uniform_matrix: &Array2<f64>,
    texture: &Option<Rgba32FImage>,
    depth: bool,
    decals: bool,
) {
    point.multiply_by_matrix(&uniform_matrix);

    point.divide_by_w(&Box::from([X, Y]));

    point.transform_to_viewport(img.width(), img.height());

    let square: Points<11> = square(*point);

    Draw::<11>::draw_points(&mut img, square, texture, depth, decals);
}
