use crate::point::Point;

#[derive(Copy, Clone)]
pub(crate) struct Position {
    x: f64,
    y: f64,
    z: f64,
    w: f64,
}

impl Position {
    pub(crate) fn new(position: Vec<f64>) -> Self {
        let x = position[0];
        let y = if position.len() > 1 { position[1] } else { 0f64 };
        let z = if position.len() > 2 { position[2] } else { 0f64 };
        let w = if position.len() > 3 { position[3] } else { 1f64 };
        Self { x, y, z, w }
    }
}

impl From<Position> for Point<4> {
    fn from(value: Position) -> Self {
        return Point::new([value.x, value.y, value.z, value.w]);
    }
}