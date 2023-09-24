use crate::axis::axis::{P, S, T, X, Y};
use crate::point::{Point, Points};
use std::mem::swap;

fn dda_setup<const DIM: usize>(a: &mut Point<DIM>, b: &mut Point<DIM>, d: usize) -> (Point<DIM>, Point<DIM>) {
    let mut a_d = a[d];
    let b_d = b[d];

    if a_d == b_d {
        return (*a, Point::zero());
    }

    if a_d > b_d {
        swap(a, b);
        a_d = b_d;
    }

    let delta = b.clone() - a.clone();

    let delta_d = delta[d];
    let s = delta / delta_d;

    let e = a_d.ceil() - a_d;
    let o = e * s;

    let p = a.clone() + o;

    return (p, s);
}

fn dda_full<const DIM: usize>(mut a: Point<DIM>, mut b: Point<DIM>, d: usize) -> Vec<Point<DIM>> {
    let res = dda_setup(&mut a, &mut b, d);

    let (mut p, s) = res;
    let mut result: Vec<Point<DIM>> = vec![];

    while p[d] < b[d] {
        result.push(p);
        p += s;
    }

    return result;
}

pub(crate) fn triangle<const DIM: usize>(p: Point<DIM>, q: Point<DIM>, r: Point<DIM>) -> Points<DIM> {
    let mut sorter = [p, q, r].clone();
    sorter.sort_by(|a, b| a[Y].partial_cmp(&b[Y]).unwrap());
    let [t, m, b] = sorter;

    let mut result: Points<DIM> = Points::<DIM>::default();
    let (mut p1, s1) = dda_setup(&mut t.clone(), &mut b.clone(), Y);
    let (mut p2, mut s2) = dda_setup(&mut t.clone(), &mut m.clone(), Y);
    while p1[Y] < m[Y] {
        let mut temp = dda_full(p2, p1, X);
        result.append(&mut temp);
        p1 += s1;
        p2 += s2;
    }

    (p2, s2) = dda_setup(&mut m.clone(), &mut b.clone(), Y);
    while p1[Y] < b[Y] {
        result.append(&mut dda_full(p2, p1, X));
        p1 += s1;
        p2 += s2;
    }

    return result;
}

pub(crate) fn square<const DIM: usize>(center: Point<DIM>) -> Points<DIM> {
    let mut result: Points<DIM> = Points::<DIM>::default();

    let radius = center[P] / 2f64;

    let mut top_left = center;
    let mut top_right = center;
    let mut bottom_left = center;
    let mut bottom_right = center;

    top_left[X] = center[X] - radius;
    top_left[Y] = center[Y] - radius;
    top_left[S] = 0f64;
    top_left[T] = 0f64;

    top_right[X] = center[X] + radius;
    top_right[Y] = center[Y] - radius;
    top_right[S] = 1f64;
    top_right[T] = 0f64;

    bottom_left[X] = center[X] - radius;
    bottom_left[Y] = center[Y] + radius;
    bottom_left[S] = 0f64;
    bottom_left[T] = 1f64;

    bottom_right[X] = center[X] + radius;
    bottom_right[Y] = center[Y] + radius;
    bottom_right[S] = 1f64;
    bottom_right[T] = 1f64;

    result.append(&mut triangle(top_left, top_right, bottom_left).0);
    result.append(&mut triangle(top_right, bottom_right, bottom_left).0);

    return result;
}
