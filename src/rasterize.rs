use crate::axis::axis::{X, Y};
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

pub(crate) fn scanline<const DIM: usize>(p: Point<DIM>, q: Point<DIM>, r: Point<DIM>) -> Points<DIM> {
    let mut sorter = [p, q, r].clone();
    sorter.sort_by(|a, b| a[Y].partial_cmp(&b[Y]).unwrap());
    let [t, m, b] = sorter;

    let mut result: Points<DIM> = Points::<DIM>(vec![]);
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
