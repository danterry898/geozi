use std::str::{FromStr, Lines, SplitWhitespace};

use crate::expr::{self, Expr};
use crate::geometry::{AngleKind, Circle, DirectedLine, LineRef, Point, Scene};
use crate::win32::*;

const OFN_OVERWRITEPROMPT: DWORD = 0x00000002;
const OFN_PATHMUSTEXIST: DWORD = 0x00000800;
const OFN_FILEMUSTEXIST: DWORD = 0x00001000;
const OFN_EXPLORER: DWORD = 0x00080000;

fn filter_wide(entries: &[(&str, &str)]) -> Vec<u16> {
    let mut buf = Vec::new();
    for (desc, pattern) in entries {
        buf.extend(desc.encode_utf16());
        buf.push(0);
        buf.extend(pattern.encode_utf16());
        buf.push(0);
    }
    buf.push(0);
    buf
}

fn pick_path(owner: HWND, save: bool) -> Option<String> {
    unsafe {
        let filter = filter_wide(&[("Geozi files", "*.gsp")]);
        let def_ext = to_wide("gsp");
        let mut file_buf: [u16; 260] = [0; 260];
        let mut ofn: OPENFILENAMEW = std::mem::zeroed();
        ofn.lStructSize = std::mem::size_of::<OPENFILENAMEW>() as u32;
        ofn.hwndOwner = owner;
        ofn.lpstrFilter = filter.as_ptr();
        ofn.lpstrFile = file_buf.as_mut_ptr();
        ofn.nMaxFile = 260;

        let picked = if save {
            ofn.lpstrDefExt = def_ext.as_ptr();
            ofn.Flags = OFN_OVERWRITEPROMPT | OFN_EXPLORER;
            GetSaveFileNameW(&mut ofn)
        } else {
            ofn.Flags = OFN_PATHMUSTEXIST | OFN_FILEMUSTEXIST | OFN_EXPLORER;
            GetOpenFileNameW(&mut ofn)
        };
        if picked == 0 {
            return None;
        }

        let len = file_buf.iter().position(|&c| c == 0).unwrap_or(0);
        Some(String::from_utf16_lossy(&file_buf[..len]))
    }
}

pub fn pick_save_path(owner: HWND) -> Option<String> {
    pick_path(owner, true)
}

pub fn pick_open_path(owner: HWND) -> Option<String> {
    pick_path(owner, false)
}

fn write_line_ref(out: &mut String, r: &LineRef) {
    match r {
        LineRef::Segment(i) => out.push_str(&format!("SEG {}", i)),
        LineRef::Line(i) => out.push_str(&format!("LINE {}", i)),
    }
}

pub fn write_scene(path: &str, scene: &Scene) -> bool {
    let mut out = String::new();
    out.push_str("GEOZI1\n");

    out.push_str(&format!("POINTS {}\n", scene.points.len()));
    for p in &scene.points {
        out.push_str(&format!("{} {}\n", p.x, p.y));
    }

    out.push_str(&format!("CIRCLES {}\n", scene.circles.len()));
    for c in &scene.circles {
        match c {
            Circle::TwoPoint(a, b) => out.push_str(&format!("TWO {} {}\n", a, b)),
            Circle::Radius(a, r) => out.push_str(&format!("RADIUS {} {}\n", a, r)),
        }
    }

    out.push_str(&format!("SEGMENTS {}\n", scene.segments.len()));
    for (a, b) in &scene.segments {
        out.push_str(&format!("{} {}\n", a, b));
    }

    out.push_str(&format!("LINES {}\n", scene.lines.len()));
    for (a, b) in &scene.lines {
        out.push_str(&format!("{} {}\n", a, b));
    }

    out.push_str(&format!("DIRECTED {}\n", scene.directed_lines.len()));
    for dl in &scene.directed_lines {
        out.push_str(&format!("{} {} {}\n", dl.anchor, dl.dx, dl.dy));
    }

    out.push_str(&format!("POLYGONS {}\n", scene.polygons.len()));
    for poly in &scene.polygons {
        out.push_str(&poly.len().to_string());
        for idx in poly {
            out.push(' ');
            out.push_str(&idx.to_string());
        }
        out.push('\n');
    }

    out.push_str(&format!("ANGLES {}\n", scene.angles.len()));
    for a in &scene.angles {
        match a {
            AngleKind::ThreePoints(p0, p1, p2) => out.push_str(&format!("THREE {} {} {}\n", p0, p1, p2)),
            AngleKind::TwoLines(l1, l2) => {
                out.push_str("TWOLINES ");
                write_line_ref(&mut out, l1);
                out.push(' ');
                write_line_ref(&mut out, l2);
                out.push('\n');
            }
        }
    }

    out.push_str(&format!("STROKES {}\n", scene.strokes.len()));
    for (color, pts) in &scene.strokes {
        out.push_str(&format!("{} {}", color, pts.len()));
        for (x, y) in pts {
            out.push_str(&format!(" {} {}", x, y));
        }
        out.push('\n');
    }

    out.push_str(&format!("FUNCTIONS {}\n", scene.functions.len()));
    for (text, _) in &scene.functions {
        out.push_str(text);
        out.push('\n');
    }

    std::fs::write(path, out).is_ok()
}

fn read_count_header(lines: &mut Lines, expected: &str) -> Option<usize> {
    let line = lines.next()?;
    let mut parts = line.split_whitespace();
    if parts.next()? != expected {
        return None;
    }
    parts.next()?.parse().ok()
}

fn next_num<T: FromStr>(parts: &mut SplitWhitespace) -> Option<T> {
    parts.next()?.parse().ok()
}

/// Reads a "TAG n" header followed by n lines, each parsed by parse_line.
fn read_section<T>(
    lines: &mut Lines,
    tag: &str,
    mut parse_line: impl FnMut(&str) -> Option<T>,
) -> Option<Vec<T>> {
    let count = read_count_header(lines, tag)?;
    (0..count).map(|_| parse_line(lines.next()?)).collect()
}

fn read_points(lines: &mut Lines) -> Option<Vec<Point>> {
    read_section(lines, "POINTS", |line| {
        let mut parts = line.split_whitespace();
        Some(Point { x: next_num(&mut parts)?, y: next_num(&mut parts)? })
    })
}

fn read_circles(lines: &mut Lines) -> Option<Vec<Circle>> {
    read_section(lines, "CIRCLES", |line| {
        let mut parts = line.split_whitespace();
        match parts.next()? {
            "TWO" => Some(Circle::TwoPoint(next_num(&mut parts)?, next_num(&mut parts)?)),
            "RADIUS" => Some(Circle::Radius(next_num(&mut parts)?, next_num(&mut parts)?)),
            _ => None,
        }
    })
}

fn read_pairs(lines: &mut Lines, tag: &str) -> Option<Vec<(usize, usize)>> {
    read_section(lines, tag, |line| {
        let mut parts = line.split_whitespace();
        Some((next_num(&mut parts)?, next_num(&mut parts)?))
    })
}

fn read_directed(lines: &mut Lines) -> Option<Vec<DirectedLine>> {
    read_section(lines, "DIRECTED", |line| {
        let mut parts = line.split_whitespace();
        Some(DirectedLine {
            anchor: next_num(&mut parts)?,
            dx: next_num(&mut parts)?,
            dy: next_num(&mut parts)?,
        })
    })
}

fn read_polygons(lines: &mut Lines) -> Option<Vec<Vec<usize>>> {
    read_section(lines, "POLYGONS", |line| {
        let mut parts = line.split_whitespace();
        let count: usize = next_num(&mut parts)?;
        (0..count).map(|_| next_num(&mut parts)).collect()
    })
}

fn parse_line_ref(parts: &mut SplitWhitespace) -> Option<LineRef> {
    let kind = parts.next()?;
    let index = next_num(parts)?;
    match kind {
        "SEG" => Some(LineRef::Segment(index)),
        "LINE" => Some(LineRef::Line(index)),
        _ => None,
    }
}

fn read_angles(lines: &mut Lines) -> Option<Vec<AngleKind>> {
    read_section(lines, "ANGLES", |line| {
        let mut parts = line.split_whitespace();
        match parts.next()? {
            "THREE" => Some(AngleKind::ThreePoints(
                next_num(&mut parts)?,
                next_num(&mut parts)?,
                next_num(&mut parts)?,
            )),
            "TWOLINES" => {
                let first = parse_line_ref(&mut parts)?;
                let second = parse_line_ref(&mut parts)?;
                Some(AngleKind::TwoLines(first, second))
            }
            _ => None,
        }
    })
}

fn read_strokes(lines: &mut Lines) -> Option<Vec<(u32, Vec<(f64, f64)>)>> {
    read_section(lines, "STROKES", |line| {
        let mut parts = line.split_whitespace();
        let color: u32 = next_num(&mut parts)?;
        let count: usize = next_num(&mut parts)?;
        let points = (0..count)
            .map(|_| Some((next_num(&mut parts)?, next_num(&mut parts)?)))
            .collect::<Option<Vec<(f64, f64)>>>()?;
        Some((color, points))
    })
}

fn read_functions(lines: &mut Lines) -> Option<Vec<(String, Expr)>> {
    if lines.clone().next().is_none() {
        return Some(Vec::new());
    }
    read_section(lines, "FUNCTIONS", |line| {
        expr::parse(line).ok().map(|e| (line.to_string(), e))
    })
}

pub fn read_scene(path: &str) -> Option<Scene> {
    let content = std::fs::read_to_string(path).ok()?;
    let mut lines = content.lines();

    lines.next()?;

    Some(Scene {
        points: read_points(&mut lines)?,
        circles: read_circles(&mut lines)?,
        segments: read_pairs(&mut lines, "SEGMENTS")?,
        lines: read_pairs(&mut lines, "LINES")?,
        directed_lines: read_directed(&mut lines)?,
        polygons: read_polygons(&mut lines)?,
        angles: read_angles(&mut lines)?,
        strokes: read_strokes(&mut lines)?,
        functions: read_functions(&mut lines)?,
    })
}