#![allow(clippy::too_many_arguments)]

use std::ops::RangeInclusive;

#[derive(Clone, Copy)]
pub struct View {
    pub width: i32,
    pub height: i32,
    pub scale: f64,
    pub offset_x: f64,
    pub offset_y: f64,
}

impl View {
    pub fn to_screen(&self, x: f64, y: f64) -> (i32, i32) {
        let sx = x * self.scale + self.offset_x + self.width as f64 / 2.0;
        let sy = self.height as f64 / 2.0 - (y * self.scale + self.offset_y);
        (sx.round() as i32, sy.round() as i32)
    }

    pub fn to_world(&self, sx: i32, sy: i32) -> (f64, f64) {
        let x = (sx as f64 - self.width as f64 / 2.0 - self.offset_x) / self.scale;
        let y = (self.height as f64 / 2.0 - sy as f64 - self.offset_y) / self.scale;
        (x, y)
    }

    pub fn visible_cells(&self, top: i32, step: f64) -> (RangeInclusive<i64>, RangeInclusive<i64>) {
        let (x_left, y_top) = self.to_world(0, top);
        let (x_right, y_bottom) = self.to_world(self.width, self.height);
        let first = |a: f64, b: f64| (a.min(b) / step).floor() as i64;
        let last = |a: f64, b: f64| (a.max(b) / step).ceil() as i64;
        (
            first(x_left, x_right)..=last(x_left, x_right),
            first(y_top, y_bottom)..=last(y_top, y_bottom),
        )
    }
}

pub fn put_pixel(buffer: &mut [u32], width: i32, height: i32, x: i32, y: i32, color: u32) {
    if x < 0 || y < 0 || x >= width || y >= height {
        return;
    }
    buffer[(y * width + x) as usize] = color;
}

// fill the rectangle [x0, x1) × [y0, y1) with the given color.
pub fn fill_rect(buffer: &mut [u32], width: i32, height: i32, x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
    for y in y0..y1 {
        for x in x0..x1 {
            put_pixel(buffer, width, height, x, y, color);
        }
    }
}

pub fn draw_line(buffer: &mut [u32], width: i32, height: i32, x0: i32, y0: i32, x1: i32, y1: i32, color: u32) {
    let mut x0 = x0;
    let mut y0 = y0;
    let dx = (x1 - x0).abs();
    let dy = -(y1 - y0).abs();
    let sx = if x0 < x1 { 1 } else { -1 };
    let sy = if y0 < y1 { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        put_pixel(buffer, width, height, x0, y0, color);
        if x0 == x1 && y0 == y1 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x0 += sx;
        }
        if e2 <= dx {
            err += dx;
            y0 += sy;
        }
    }
}

pub fn draw_thick_line(buffer: &mut [u32], width: i32, height: i32, x0: i32, y0: i32, x1: i32, y1: i32, radius: i32, color: u32) {
    draw_line(buffer, width, height, x0, y0, x1, y1, color);
    for d in 1..=radius {
        draw_line(buffer, width, height, x0 - d, y0, x1 - d, y1, color);
        draw_line(buffer, width, height, x0 + d, y0, x1 + d, y1, color);
        draw_line(buffer, width, height, x0, y0 - d, x1, y1 - d, color);
        draw_line(buffer, width, height, x0, y0 + d, x1, y1 + d, color);
    }
}

pub fn draw_point(buffer: &mut [u32], width: i32, height: i32, x: i32, y: i32, color: u32) {
    for dy in -3..=3 {
        for dx in -3..=3 {
            if dx * dx + dy * dy <= 9 {
                put_pixel(buffer, width, height, x + dx, y + dy, color);
            }
        }
    }
}

pub fn draw_circle(buffer: &mut [u32], width: i32, height: i32, cx: i32, cy: i32, radius: i32, color: u32) {
    let mut x = radius;
    let mut y = 0;
    let mut err = 0;

    while x >= y {
        put_pixel(buffer, width, height, cx + x, cy + y, color);
        put_pixel(buffer, width, height, cx + y, cy + x, color);
        put_pixel(buffer, width, height, cx - y, cy + x, color);
        put_pixel(buffer, width, height, cx - x, cy + y, color);
        put_pixel(buffer, width, height, cx - x, cy - y, color);
        put_pixel(buffer, width, height, cx - y, cy - x, color);
        put_pixel(buffer, width, height, cx + y, cy - x, color);
        put_pixel(buffer, width, height, cx + x, cy - y, color);

        y += 1;
        if err <= 0 {
            err += 2 * y + 1;
        }
        if err > 0 {
            x -= 1;
            err -= 2 * x + 1;
        }
    }
}

pub fn draw_polygon_fill(buffer: &mut [u32], width: i32, height: i32, points: &[(i32, i32)], color: u32) {
    if points.len() < 3 {
        return;
    }

    let min_y = points.iter().map(|p| p.1).min().unwrap().max(0);
    let max_y = points.iter().map(|p| p.1).max().unwrap().min(height - 1);
    let n = points.len();

    for y in min_y..=max_y {
        let mut xs = Vec::new();
        for i in 0..n {
            let (x0, y0) = points[i];
            let (x1, y1) = points[(i + 1) % n];
            if (y0 <= y && y1 > y) || (y1 <= y && y0 > y) {
                let t = (y - y0) as f64 / (y1 - y0) as f64;
                let x = x0 as f64 + t * (x1 - x0) as f64;
                xs.push(x.round() as i32);
            }
        }
        xs.sort();

        let mut i = 0;
        while i + 1 < xs.len() {
            for x in xs[i]..xs[i + 1] {
                put_pixel(buffer, width, height, x, y, color);
            }
            i += 2;
        }
    }
}

pub fn blend(fg: u32, bg: u32, alpha: f64) -> u32 {
    let mix = |shift: u32| {
        let f = ((fg >> shift) & 0xff) as f64;
        let b = ((bg >> shift) & 0xff) as f64;
        (f * alpha + b * (1.0 - alpha)) as u32
    };
    (mix(16) << 16) | (mix(8) << 8) | mix(0)
}

// gap a2 - a1 brought back to ]-π, π].
pub fn angle_diff(a1: f64, a2: f64) -> f64 {
    let mut diff = a2 - a1;
    while diff <= -std::f64::consts::PI {
        diff += 2.0 * std::f64::consts::PI;
    }
    while diff > std::f64::consts::PI {
        diff -= 2.0 * std::f64::consts::PI;
    }
    diff
}

pub fn draw_angle_arc(buffer: &mut [u32], width: i32, height: i32, vx: i32, vy: i32, a1: f64, a2: f64, radius: i32, color: u32) {
    let diff = angle_diff(a1, a2);
    let steps = 12;
    let mut prev_x = vx + (radius as f64 * a1.cos()) as i32;
    let mut prev_y = vy + (radius as f64 * a1.sin()) as i32;
    for i in 1..=steps {
        let t = a1 + diff * (i as f64 / steps as f64);
        let x = vx + (radius as f64 * t.cos()) as i32;
        let y = vy + (radius as f64 * t.sin()) as i32;
        draw_line(buffer, width, height, prev_x, prev_y, x, y, color);
        prev_x = x;
        prev_y = y;
    }
}

pub fn draw_infinite_line(buffer: &mut [u32], view: &View, ax: f64, ay: f64, bx: f64, by: f64, color: u32) {
    let dx = bx - ax;
    let dy = by - ay;
    if dx.abs() < 1e-9 && dy.abs() < 1e-9 {
        return;
    }
    let len = (dx * dx + dy * dy).sqrt();
    let ux = dx / len;
    let uy = dy / len;
    let extend = (view.width + view.height) as f64 / view.scale + 10.0;
    let (sx0, sy0) = view.to_screen(ax - ux * extend, ay - uy * extend);
    let (sx1, sy1) = view.to_screen(ax + ux * extend, ay + uy * extend);
    draw_line(buffer, view.width, view.height, sx0, sy0, sx1, sy1, color);
}

pub fn grid_step(scale: f64) -> f64 {
    let target_px = 60.0;
    let raw = (target_px / scale).max(0.0001);
    let magnitude = 10f64.powf(raw.log10().floor());
    let residual = raw / magnitude;
    let nice = if residual < 1.5 {
        1.0
    } else if residual < 3.5 {
        2.0
    } else if residual < 7.5 {
        5.0
    } else {
        10.0
    };
    nice * magnitude
}

pub fn draw_grid_lines(buffer: &mut [u32], view: &View, top: i32, step: f64, color: u32) {
    let (columns, rows) = view.visible_cells(top, step);

    for i in columns {
        let (sx, _) = view.to_screen(i as f64 * step, 0.0);
        if sx >= 0 && sx < view.width {
            draw_line(buffer, view.width, view.height, sx, top, sx, view.height - 1, color);
        }
    }

    for i in rows {
        let (_, sy) = view.to_screen(0.0, i as f64 * step);
        if sy >= top && sy < view.height {
            draw_line(buffer, view.width, view.height, 0, sy, view.width - 1, sy, color);
        }
    }
}

pub fn draw_axes(buffer: &mut [u32], view: &View, top: i32, color: u32) {
    let (ox, oy) = view.to_screen(0.0, 0.0);
    if oy >= top && oy < view.height {
        draw_line(buffer, view.width, view.height, 0, oy, view.width - 1, oy, color);
    }
    if ox >= 0 && ox < view.width {
        draw_line(buffer, view.width, view.height, ox, top, ox, view.height - 1, color);
    }
}

pub fn draw_graduations(buffer: &mut [u32], view: &View, top: i32, step: f64, color: u32) {
    let (ox, oy) = view.to_screen(0.0, 0.0);
    let (columns, rows) = view.visible_cells(top, step);

    if oy >= top && oy < view.height {
        for i in columns {
            if i == 0 {
                continue;
            }
            let (sx, _) = view.to_screen(i as f64 * step, 0.0);
            if sx >= 0 && sx < view.width {
                draw_line(buffer, view.width, view.height, sx, oy - 5, sx, oy + 5, color);
            }
        }
    }

    if ox >= 0 && ox < view.width {
        for i in rows {
            if i == 0 {
                continue;
            }
            let (_, sy) = view.to_screen(0.0, i as f64 * step);
            if sy >= top && sy < view.height {
                draw_line(buffer, view.width, view.height, ox - 5, sy, ox + 5, sy, color);
            }
        }
    }
}