use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, Range, Sub};
use std::vec::IntoIter;
use blend_srgb::convert::rgb_to_srgb;
use image::Rgba;
use crate::axis::axis::{R, G, B, A, W};
use crate::color::Color;
use crate::position::Position;

#[derive(Clone)]
pub(crate) struct Points<const DIM: usize> (pub(crate) Vec<Point<DIM>>);

impl<const DIM: usize> Index<usize> for Points<DIM> {
    type Output = Point<DIM>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<const DIM: usize> IntoIterator for Points<DIM> {
    type Item = Point<DIM>;
    type IntoIter = IntoIter<Point<DIM>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<const DIM: usize> Points<DIM> {
    pub(crate) fn append(&mut self, other: &mut Vec<Point<DIM>>) {
        self.0.append(other);
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn divide_by_w(&mut self, wd: usize, fields: &Box<[usize]>) {
        for point in &mut self.0 {
            point.divide_by_w(wd, fields);
        }
    }

    pub(crate) fn transform_to_viewport(&mut self, xd: usize, yd: usize, width: u32, height: u32) {
        for point in &mut self.0 {
            point.transform_to_viewport(xd, yd, width, height);
        }
    }

    pub(crate) fn undivide_by_w(&mut self, wd: usize, fields: &Box<[usize]>) {
        for point in &mut self.0 {
            point.undivide_by_w(wd, fields);
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Point<const DIM: usize> {
    data: [f64; DIM],
}

impl From<(Position, Color)> for Point<8> {
    fn from(value: (Position, Color)) -> Self {
        Self { data: <[f64; 8]>::try_from(Point::from(value.0).data().into_iter().chain(Point::from(value.1).data().into_iter()).collect::<Vec<f64>>()).unwrap() }
    }
}

impl<const DIM: usize> Display for Point<DIM> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Point: ")?;
        for i in 0..self.data.len() - 1 {
            write!(f, "{}, ", self.data[i])?;
        }
        if let Some(last) = self.data.last() {
            write!(f, "{}", last)?;
        }
        Ok(())
    }
}

impl<const DIM: usize> Display for Points<DIM> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Vector:\n")?;
        for p in &self.0 {
            write!(f, "{}\n", p)?;
        }
        Ok(())
    }
}

impl<const DIM: usize> Point<DIM> {
    pub(crate) fn new(data: [f64; DIM]) -> Self {
        Self { data }
    }

    pub(crate) fn zero() -> Self {
        Self { data: [0f64; DIM] }
    }

    pub(crate) fn data(self) -> [f64; DIM] {
        return self.data;
    }

    pub(crate) fn pixel(self) -> Rgba<u8> {
        Rgba(<[u8; 4]>::try_from(self.data[R..=A].iter().map(|&a| (a * 255f64) as u8).collect::<Vec<u8>>()).unwrap())
    }

    pub(crate) fn pixel_s_rgb(self) -> Rgba<u8> {
        let r = (rgb_to_srgb(self.data[R] as f32) * 255f32) as u8;
        let g = (rgb_to_srgb(self.data[G] as f32) * 255f32) as u8;
        let b = (rgb_to_srgb(self.data[B] as f32) * 255f32) as u8;
        Rgba([r, g, b, (self.data[A] * 255f64) as u8])
    }

    pub(crate) fn divide_by_w(&mut self, wd: usize, fields: &Box<[usize]>) {
        let w = self[wd];
        for &field in fields.into_iter() {
            self[field] /= w;
        }
        self[wd] = 1f64 / w;
    }

    pub(crate) fn transform_to_viewport(&mut self, xd: usize, yd: usize, width: u32, height: u32) {
        let x = self[xd];
        let y = self[yd];
        self[xd] = (x + 1f64) * width as f64 / 2f64;
        self[yd] = (y + 1f64) * height as f64 / 2f64;
    }

    pub(crate) fn undivide_by_w(&mut self, wd: usize, fields: &Box<[usize]>) {
        let un_w = self[wd];
        for &field in fields.into_iter() {
            self[field] /= un_w;
        }
        self[wd] = 1f64 / un_w;
    }
}

impl<const DIM: usize> Index<usize> for Point<DIM> {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<const DIM: usize> Index<Range<usize>> for Point<DIM> {
    type Output = [f64];

    fn index(&self, index: Range<usize>) -> &Self::Output {
        &self.data[index]
    }
}

impl<const DIM: usize> IndexMut<usize> for Point<DIM> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl<const DIM: usize> Sub for Point<DIM> {
    type Output = Point<DIM>;

    fn sub(self, rhs: Point<DIM>) -> Self::Output {
        return Self::Output { data: <[f64; DIM]>::try_from((self.data).into_iter().zip(rhs.data).map(|(a, b)| a - b).collect::<Vec<f64>>()).unwrap() };
    }
}

impl<const DIM: usize> Div<f64> for Point<DIM> {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        return Self { data: self.data.map(|a| a / rhs) };
    }
}

impl<const DIM: usize> DivAssign<f64> for Point<DIM> {
    fn div_assign(&mut self, rhs: f64) {
        self.data = (*self / rhs).data;
    }
}

impl<const DIM: usize> Mul<Point<DIM>> for f64 {
    type Output = Point<DIM>;

    fn mul(self, rhs: Point<DIM>) -> Self::Output {
        return Self::Output { data: rhs.data.map(|a| a * self) };
    }
}

impl<const DIM: usize> Add<Point<DIM>> for Point<DIM> {
    type Output = Point<DIM>;

    fn add(self, rhs: Point<DIM>) -> Self::Output {
        return Self::Output { data: <[f64; DIM]>::try_from(self.data.into_iter().zip(rhs.data).map(|(a, b)| a + b).collect::<Vec<f64>>()).unwrap() };
    }
}

impl<const DIM: usize> AddAssign<Point<DIM>> for Point<DIM> {
    fn add_assign(&mut self, rhs: Point<DIM>) {
        self.data = (*self + rhs).data;
    }
}
