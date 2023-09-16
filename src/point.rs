use crate::axis::axis::{A, B, G, R, W, X, Y, Z};
use crate::color::Color;
use crate::position::Position;
use image::Rgba;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Add, AddAssign, Div, DivAssign, Index, IndexMut, Mul, Range, Sub};

#[derive(Clone)]
pub(crate) struct Points<const DIM: usize>(pub(crate) Vec<Point<DIM>>);

impl<const DIM: usize> Index<usize> for Points<DIM> {
    type Output = Point<DIM>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<const DIM: usize> IntoIterator for Points<DIM> {
    type Item = Point<DIM>;
    type IntoIter = std::vec::IntoIter<Point<DIM>>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<const DIM: usize> Points<DIM> {
    pub(crate) fn append(&mut self, other: &mut Vec<Point<DIM>>) {
        self.0.append(other);
    }

    pub(crate) fn push(&mut self, value: Point<DIM>) {
        self.0.push(value);
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }

    pub(crate) fn is_back_face(&self) -> bool {
        assert_eq!(self.len(), 3);

        // Signed area of triangle 0-1-2 with counter-clockwise == positive
        return (self[1][X] - self[0][X]) * (self[2][Y] - self[0][Y])
            - (self[2][X] - self[0][X]) * (self[1][Y] - self[0][Y])
            > 0f64;
    }

    pub(crate) fn divide_by_w(&mut self, fields: &Box<[usize]>) {
        for point in &mut self.0 {
            point.divide_by_w(fields);
        }
    }

    pub(crate) fn transform_to_viewport(&mut self, width: u32, height: u32) {
        for point in &mut self.0 {
            point.transform_to_viewport(width, height);
        }
    }

    pub(crate) fn undivide_by_w(&mut self, fields: &Box<[usize]>) {
        for point in &mut self.0 {
            point.undivide_by_w(fields);
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct Point<const DIM: usize> {
    data: [f64; DIM],
}

impl From<(Position, Color)> for Point<8> {
    fn from(value: (Position, Color)) -> Self {
        Self {
            data: <[f64; 8]>::try_from(
                Point::from(value.0)
                    .data()
                    .into_iter()
                    .chain(Point::from(value.1).data().into_iter())
                    .collect::<Vec<f64>>(),
            )
            .unwrap(),
        }
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

    pub(crate) fn pixel(self) -> Rgba<f32> {
        Rgba(
            <[f32; 4]>::try_from(
                self.data[R..=A]
                    .iter()
                    .map(|&a| a as f32 * 255f32)
                    .collect::<Vec<f32>>(),
            )
            .unwrap(),
        )
    }

    pub(crate) fn divide_by_w(&mut self, fields: &Box<[usize]>) {
        let w = self[W];
        for &field in fields.into_iter() {
            self[field] /= w;
        }
        self[W] = 1f64 / w;
    }

    pub(crate) fn transform_to_viewport(&mut self, width: u32, height: u32) {
        let x = self[X];
        let y = self[Y];
        self[X] = (x + 1f64) * width as f64 / 2f64;
        self[Y] = (y + 1f64) * height as f64 / 2f64;
    }

    pub(crate) fn undivide_by_w(&mut self, fields: &Box<[usize]>) {
        let un_w = self[W];
        for &field in fields.into_iter() {
            self[field] /= un_w;
        }
        self[W] = 1f64 / un_w;
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
        return Self::Output {
            data: <[f64; DIM]>::try_from(
                (self.data)
                    .into_iter()
                    .zip(rhs.data)
                    .map(|(a, b)| a - b)
                    .collect::<Vec<f64>>(),
            )
            .unwrap(),
        };
    }
}

impl<const DIM: usize> Div<f64> for Point<DIM> {
    type Output = Self;

    fn div(self, rhs: f64) -> Self::Output {
        return Self {
            data: self.data.map(|a| a / rhs),
        };
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
        return Self::Output {
            data: rhs.data.map(|a| a * self),
        };
    }
}

impl<const DIM: usize> Add<Point<DIM>> for Point<DIM> {
    type Output = Point<DIM>;

    fn add(self, rhs: Point<DIM>) -> Self::Output {
        return Self::Output {
            data: <[f64; DIM]>::try_from(
                self.data
                    .into_iter()
                    .zip(rhs.data)
                    .map(|(a, b)| a + b)
                    .collect::<Vec<f64>>(),
            )
            .unwrap(),
        };
    }
}

impl<const DIM: usize> AddAssign<Point<DIM>> for Point<DIM> {
    fn add_assign(&mut self, rhs: Point<DIM>) {
        self.data = (*self + rhs).data;
    }
}
