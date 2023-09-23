use crate::axis::axis::{A, B, G, R, S, T, X, Y, Z};
use crate::depth_image::Image;
use crate::point::Points;
use crate::rasterize::scanline;
use image::{Rgba, Rgba32FImage};
use ndarray::Array2;
use palette::rgb::Rgb;
use palette::Srgb;

fn overlay_pixels(cur_pixel: Rgba<f32>, pixel: Rgba<f32>) -> [f32; 4] {
    let [r_s, g_s, b_s, a_s] = pixel.0.map(|a| a / 255f32);
    let [r_d, g_d, b_d, a_d] = cur_pixel.0.map(|a| a / 255f32);

    let a = a_s + a_d * (1f32 - a_s);
    let r = a_s / a * r_s + (1f32 - a_s) * a_d / a * r_d;
    let g = a_s / a * g_s + (1f32 - a_s) * a_d / a * g_d;
    let b = a_s / a * b_s + (1f32 - a_s) * a_d / a * b_d;

    return [r, g, b, a].map(|a| a * 255f32);
}

fn draw_points(
    img: &mut Image,
    points: Points<10>,
    texture: &Option<Rgba32FImage>,
    depth: bool,
    s_rgb: bool,
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
                let mut cur_pixel = img.get_pixel(x, y).clone();
                if s_rgb {
                    let rgb_pixel =
                        Srgb::from_components((cur_pixel[0] / 255f32, cur_pixel[1] / 255f32, cur_pixel[2] / 255f32))
                            .into_linear();
                    cur_pixel[0] = rgb_pixel.red * 255f32;
                    cur_pixel[1] = rgb_pixel.green * 255f32;
                    cur_pixel[2] = rgb_pixel.blue * 255f32;
                }

                [pixel[0], pixel[1], pixel[2], pixel[3]] = overlay_pixels(cur_pixel, pixel);

                if let Some(texture) = texture {
                    let x = (point[S] * texture.width() as f64) as u32;
                    let y = (point[T] * texture.height() as f64) as u32;
                    let mut temp = texture.get_pixel(x, y).clone();
                    let texel = Srgb::from_components((temp[0], temp[1], temp[2])).into_linear();
                    temp[0] = texel.red * 255f32;
                    temp[1] = texel.green * 255f32;
                    temp[2] = texel.blue * 255f32;
                    temp[3] *= 255f32;
                    if decals {
                        [pixel[0], pixel[1], pixel[2], pixel[3]] = overlay_pixels(pixel, temp);
                    } else {
                        pixel = temp;
                    }
                }

                if s_rgb {
                    let srgb_pixel = Srgb::<f32>::from_linear(Rgb::from_components((
                        pixel[0] / 255f32,
                        pixel[1] / 255f32,
                        pixel[2] / 255f32,
                    )));
                    pixel[0] = srgb_pixel.red * 255f32;
                    pixel[1] = srgb_pixel.green * 255f32;
                    pixel[2] = srgb_pixel.blue * 255f32;
                }

                img.put_pixel(x, y, pixel, if depth { Some(point[Z]) } else { None });
            }
        }
    }
}

pub(crate) fn draw_triangle(
    mut img: &mut Image,
    points: &mut Points<10>,
    uniform_matrix: &Array2<f64>,
    texture: &Option<Rgba32FImage>,
    depth: bool,
    s_rgb: bool,
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

    let mut triangle = scanline(points[0], points[1], points[2]);

    if hyp {
        triangle.undivide_by_w(&Box::from([Z, R, G, B, A, S, T]));
    }

    triangle.wrap_texcoords();

    draw_points(&mut img, triangle, texture, depth, s_rgb, decals);
}
