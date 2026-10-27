use crate::expr::Expr;

#[derive(Clone, Copy)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

pub fn distance(a: Point, b: Point) -> f64 {
    let dx = a.x - b.x;
    let dy = a.y - b.y;
    (dx * dx + dy * dy).sqrt()
}

pub fn point_segment_distance(p: Point, a: Point, b: Point) -> f64 {
    let dx = b.x - a.x;
    let dy = b.y - a.y;
    let len_sq = dx * dx + dy * dy;
    if len_sq < 1e-12 {
        return distance(p, a);
    }
    let t = (((p.x - a.x) * dx + (p.y - a.y) * dy) / len_sq).clamp(0.0, 1.0);
    let proj = Point { x: a.x + t * dx, y: a.y + t * dy };
    distance(p, proj)
}

pub fn point_line_distance(p: Point, a: Point, dx: f64, dy: f64) -> f64 {
    let len = (dx * dx + dy * dy).sqrt();
    if len < 1e-12 {
        return distance(p, a);
    }
    let ux = dx / len;
    let uy = dy / len;
    let proj_len = (p.x - a.x) * ux + (p.y - a.y) * uy;
    let proj = Point { x: a.x + ux * proj_len, y: a.y + uy * proj_len };
    distance(p, proj)
}

pub fn point_in_polygon(p: Point, poly: &[Point]) -> bool {
    let mut inside = false;
    let n = poly.len();
    let mut j = n - 1;
    for i in 0..n {
        let pi = poly[i];
        let pj = poly[j];
        if (pi.y > p.y) != (pj.y > p.y)
            && p.x < (pj.x - pi.x) * (p.y - pi.y) / (pj.y - pi.y) + pi.x
        {
            inside = !inside;
        }
        j = i;
    }
    inside
}

pub fn polygon_area(poly: &[Point]) -> f64 {
    let n = poly.len();
    let mut sum = 0.0;
    for i in 0..n {
        let a = poly[i];
        let b = poly[(i + 1) % n];
        sum += a.x * b.y - b.x * a.y;
    }
    (sum / 2.0).abs()
}

pub fn regular_polygon(a: Point, b: Point, sides: usize) -> Vec<Point> {
    let mut verts = vec![a, b];
    let angle = 2.0 * std::f64::consts::PI / sides as f64;
    for _ in 2..sides {
        let prev = verts[verts.len() - 1];
        let prev2 = verts[verts.len() - 2];
        let ex = prev.x - prev2.x;
        let ey = prev.y - prev2.y;
        let nx = ex * angle.cos() - ey * angle.sin();
        let ny = ex * angle.sin() + ey * angle.cos();
        verts.push(Point { x: prev.x + nx, y: prev.y + ny });
    }
    verts
}

pub fn line_intersection(a1: Point, d1: (f64, f64), a2: Point, d2: (f64, f64)) -> Option<Point> {
    let denom = d1.0 * d2.1 - d1.1 * d2.0;
    if denom.abs() < 1e-9 {
        return None;
    }
    let t = ((a2.x - a1.x) * d2.1 - (a2.y - a1.y) * d2.0) / denom;
    Some(Point { x: a1.x + d1.0 * t, y: a1.y + d1.1 * t })
}

pub fn angle_between(v1: (f64, f64), v2: (f64, f64)) -> f64 {
    let dot = v1.0 * v2.0 + v1.1 * v2.1;
    let mag1 = (v1.0 * v1.0 + v1.1 * v1.1).sqrt();
    let mag2 = (v2.0 * v2.0 + v2.1 * v2.1).sqrt();
    if mag1 < 1e-9 || mag2 < 1e-9 {
        return 0.0;
    }
    let cos_a = (dot / (mag1 * mag2)).clamp(-1.0, 1.0);
    cos_a.acos().to_degrees()
}

#[derive(Clone)]
pub enum Circle {
    TwoPoint(usize, usize),
    Radius(usize, f64),
}

impl Circle {
    pub fn center(&self) -> usize {
        match self {
            Circle::TwoPoint(c, _) => *c,
            Circle::Radius(c, _) => *c,
        }
    }

    pub fn radius(&self, points: &[Point]) -> f64 {
        match self {
            Circle::TwoPoint(c, e) => distance(points[*c], points[*e]),
            Circle::Radius(_, r) => *r,
        }
    }

    pub fn references(&self, idx: usize) -> bool {
        match self {
            Circle::TwoPoint(c, e) => *c == idx || *e == idx,
            Circle::Radius(c, _) => *c == idx,
        }
    }

    pub fn remap_points(&mut self, f: impl Fn(usize) -> usize) {
        match self {
            Circle::TwoPoint(c, e) => {
                *c = f(*c);
                *e = f(*e);
            }
            Circle::Radius(c, _) => *c = f(*c),
        }
    }
}

#[derive(Clone, Copy)]
pub enum LineRef {
    Segment(usize),
    Line(usize),
}

#[derive(Clone)]
pub struct DirectedLine {
    pub anchor: usize,
    pub dx: f64,
    pub dy: f64,
}

#[derive(Clone, Copy)]
pub enum AngleKind {
    ThreePoints(usize, usize, usize),
    TwoLines(LineRef, LineRef),
}

/// Everything saved in a .gsp file and restored by undo
#[derive(Clone, Default)]
pub struct Scene {
    pub points: Vec<Point>,
    pub circles: Vec<Circle>,
    pub segments: Vec<(usize, usize)>,
    pub lines: Vec<(usize, usize)>,
    pub directed_lines: Vec<DirectedLine>,
    pub polygons: Vec<Vec<usize>>,
    pub angles: Vec<AngleKind>,
    pub strokes: Vec<(u32, Vec<(f64, f64)>)>,

    pub functions: Vec<(String, Expr)>,
}

pub fn index_after_removal(index: usize, removed: &[usize]) -> usize {
    index - removed.iter().filter(|&&r| r < index).count()
}

impl Scene {
    pub fn delete(&mut self, points: &[usize], circles: &[usize]) {
        let mut n = 0;
        self.circles.retain(|c| {
            let doomed = circles.contains(&n) || points.iter().any(|&p| c.references(p));
            n += 1;
            !doomed
        });

        let segments_before = self.segments.len();
        let lines_before = self.lines.len();
        let touches = |&(a, b): &(usize, usize)| points.contains(&a) || points.contains(&b);
        self.segments.retain(|s| !touches(s));
        self.lines.retain(|l| !touches(l));
        let lines_changed =
            self.segments.len() != segments_before || self.lines.len() != lines_before;

        self.directed_lines.retain(|dl| !points.contains(&dl.anchor));
        self.polygons.retain(|poly| !poly.iter().any(|p| points.contains(p)));

        self.angles.retain(|a| match a {
            AngleKind::ThreePoints(p0, p1, p2) => {
                !(points.contains(p0) || points.contains(p1) || points.contains(p2))
            }
            AngleKind::TwoLines(..) => !lines_changed,
        });

        let mut i = 0;
        self.points.retain(|_| {
            let keep = !points.contains(&i);
            i += 1;
            keep
        });

        let shift = |i: usize| index_after_removal(i, points);
        for c in self.circles.iter_mut() {
            c.remap_points(shift);
        }
        for (a, b) in self.segments.iter_mut().chain(self.lines.iter_mut()) {
            *a = shift(*a);
            *b = shift(*b);
        }
        for dl in self.directed_lines.iter_mut() {
            dl.anchor = shift(dl.anchor);
        }
        for p in self.polygons.iter_mut().flatten() {
            *p = shift(*p);
        }
        for a in self.angles.iter_mut() {
            if let AngleKind::ThreePoints(p0, p1, p2) = a {
                *p0 = shift(*p0);
                *p1 = shift(*p1);
                *p2 = shift(*p2);
            }
        }
    }
}

pub fn point_label(index: usize) -> (String, Option<String>) {
    let letter = (b'A' + (index % 26) as u8) as char;
    let suffix = index / 26;
    if suffix == 0 {
        (letter.to_string(), None)
    } else {
        (letter.to_string(), Some(suffix.to_string()))
    }
}

pub fn point_name(index: usize) -> String {
    let (letter, suffix) = point_label(index);
    format!("{}{}", letter, suffix.unwrap_or_default())
}