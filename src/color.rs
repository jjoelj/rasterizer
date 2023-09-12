use crate::point::Point;

#[derive(Copy, Clone)]
pub(crate) struct Color {
    r: f64,
    g: f64,
    b: f64,
    a: f64,
}

impl Color {
    pub(crate) fn new(color: Vec<f64>) -> Self {
        let a = if color.len() > 3 { color[3] } else { 1f64 };
        Self { r: color[0], g: color[1], b: color[2], a: a }
    }
}

impl From<Color> for Point<4> {
    fn from(value: Color) -> Self {
        return Point::new([value.r, value.g, value.b, value.a]);
    }
}