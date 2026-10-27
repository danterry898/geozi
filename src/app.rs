use crate::control;
use crate::dialog;
use crate::expr;
use crate::geometry::{self, distance, AngleKind, Circle, DirectedLine, LineRef, Point, Scene};
use crate::render::{
    angle_diff, blend, draw_angle_arc, draw_axes, draw_circle, draw_graduations, draw_grid_lines,
    draw_infinite_line, draw_line, draw_point, draw_polygon_fill, draw_thick_line, grid_step, View,
};
use crate::storage;
use crate::theme::{palette, Palette, Theme};
use crate::ui::{
    self, draw_pen_palette, draw_slider, draw_submenu, draw_toolbar, BUTTON_SIZE, PEN_COLORS,
    SUBMENU_ITEM_HEIGHT, SUBMENU_WIDTH, SWATCH_SIZE, TOOLBAR_HEIGHT,
};
use crate::win32::*;

const VK_DELETE: usize = 0x2E;
const KEY_A: usize = 0x41;
const KEY_Z: usize = 0x5A;
const PEN_PALETTE_X: i32 = BUTTON_SIZE * Mode::Pen as i32 + SUBMENU_WIDTH;

#[derive(PartialEq, Clone, Copy)]
pub enum Mode {
    File = 0,
    Cursor = 1,
    Point = 2,
    Circle = 3,
    Polygon = 4,
    Line = 5,
    Helpers = 6,
    Angle = 7,
    Pen = 8,
    Function = 9,
}

impl Mode {
    const ALL: [Mode; 10] = [
        Mode::File,
        Mode::Cursor,
        Mode::Point,
        Mode::Circle,
        Mode::Polygon,
        Mode::Line,
        Mode::Helpers,
        Mode::Angle,
        Mode::Pen,
        Mode::Function,
    ];

    fn menu_x(self) -> i32 {
        BUTTON_SIZE * self as i32
    }

    fn submenu_labels(self) -> &'static [&'static str] {
        match self {
            Mode::File => &["New", "Save", "Open"],
            Mode::Cursor => &[],
            Mode::Point => &["Point", "Segment", "Line", "Move"],
            Mode::Circle => &["Center through Point", "Center and Radius"],
            Mode::Polygon => &["Polygon", "Regular Polygon"],
            Mode::Line => &["Parallel Line", "Perpendicular Line"],
            Mode::Helpers => &["Distance or Length", "Area"],
            Mode::Angle => &["Angle (3 Points)", "Angle (2 Lines)"],
            Mode::Pen => &["Pen", "Eraser"],
            Mode::Function => &[],
        }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum PointTool {
    Place = 0,
    Segment = 1,
    Line = 2,
    Move = 3,
}

#[derive(PartialEq, Clone, Copy)]
pub enum CircleTool {
    Basic = 0,
    Radius = 1,
}

#[derive(PartialEq, Clone, Copy)]
pub enum PolygonTool {
    Freeform = 0,
    Regular = 1,
}

#[derive(PartialEq, Clone, Copy)]
pub enum LineTool {
    Parallel = 0,
    Perpendicular = 1,
}

#[derive(PartialEq, Clone, Copy)]
pub enum HelperTool {
    Distance = 0,
    Area = 1,
}

#[derive(PartialEq, Clone, Copy)]
pub enum AngleTool {
    ThreePoints = 0,
    TwoLines = 1,
}

#[derive(PartialEq, Clone, Copy)]
pub enum PenTool {
    Draw = 0,
    Erase = 1,
}

enum Prompt {
    Radius(usize),
    Sides(usize, usize),
    Save,
    Load,
    Function,
}

struct AngleView {
    vx: i32,
    vy: i32,
    from: f64,
    to: f64,
    degrees: f64,
}

pub struct AppState {
    pub view: View,
    pub buffer: Vec<u32>,
    pub scene: Scene,
    pub selected_points: Vec<usize>,
    pub selected_circles: Vec<usize>,
    pub selected_functions: Vec<usize>,
    pub polygon_in_progress: Vec<usize>,
    pub current_stroke: Vec<(f64, f64)>,
    pub pen_drawing: bool,
    pub pen_color: u32,
    pub pen_tool: PenTool,
    pub mode: Mode,
    pub point_tool: PointTool,
    pub circle_tool: CircleTool,
    pub polygon_tool: PolygonTool,
    pub line_tool: LineTool,
    pub helper_tool: HelperTool,
    pub angle_tool: AngleTool,
    pub theme: Theme,
    pub pending_center: Option<usize>,
    pub pending_segment: Option<usize>,
    pub pending_line: Option<usize>,
    pub moving_point: Option<usize>,
    pub pending_reference: Option<LineRef>,
    pub line_preview: Option<(f64, f64)>,
    pub pending_distance_point: Option<usize>,
    pub pending_regular: Option<usize>,
    pub pending_angle_points: Vec<usize>,
    pub pending_angle_line: Option<LineRef>,
    pub helper_text: Option<String>,
    pub show_grid: bool,
    pub show_graduations: bool,
    pub dragging: bool,
    pub drag_start_x: i32,
    pub drag_start_y: i32,
    pub drag_start_offset_x: f64,
    pub drag_start_offset_y: f64,
    pub undo_stack: Vec<Scene>,
}

impl AppState {
    pub fn new(width: i32, height: i32) -> Self {
        AppState {
            view: View { width, height, scale: 40.0, offset_x: 0.0, offset_y: 0.0 },
            buffer: vec![0; (width * height) as usize],
            scene: Scene::default(),
            selected_points: Vec::new(),
            selected_circles: Vec::new(),
            selected_functions: Vec::new(),
            polygon_in_progress: Vec::new(),
            current_stroke: Vec::new(),
            pen_drawing: false,
            pen_color: PEN_COLORS[0],
            pen_tool: PenTool::Draw,
            mode: Mode::Cursor,
            point_tool: PointTool::Place,
            circle_tool: CircleTool::Basic,
            polygon_tool: PolygonTool::Freeform,
            line_tool: LineTool::Parallel,
            helper_tool: HelperTool::Distance,
            angle_tool: AngleTool::ThreePoints,
            theme: Theme::Dark,
            pending_center: None,
            pending_segment: None,
            pending_line: None,
            moving_point: None,
            pending_reference: None,
            line_preview: None,
            pending_distance_point: None,
            pending_regular: None,
            pending_angle_points: Vec::new(),
            pending_angle_line: None,
            helper_text: None,
            show_grid: false,
            show_graduations: false,
            dragging: false,
            drag_start_x: 0,
            drag_start_y: 0,
            drag_start_offset_x: 0.0,
            drag_start_offset_y: 0.0,
            undo_stack: Vec::new(),
        }
    }

    pub fn resize(&mut self, width: i32, height: i32) {
        self.view.width = width;
        self.view.height = height;
        self.buffer.resize((width * height) as usize, 0);
    }

    pub fn clear_selection(&mut self) {
        self.selected_points.clear();
        self.selected_circles.clear();
        self.selected_functions.clear();
    }


    /// Forgive everything that wait the next click
    pub fn clear_pending(&mut self) {
        self.pending_center = None;
        self.pending_segment = None;
        self.pending_line = None;
        self.pending_reference = None;
        self.line_preview = None;
        self.pending_distance_point = None;
        self.pending_regular = None;
        self.pending_angle_points.clear();
        self.pending_angle_line = None;
        self.polygon_in_progress.clear();
    }

    pub fn delete_selected(&mut self) {
        let mut doomed = self.selected_points.clone();
        doomed.sort();
        doomed.dedup();
        self.scene.delete(&doomed, &self.selected_circles);

        self.polygon_in_progress.retain(|p| !doomed.contains(p));
        for p in self.polygon_in_progress.iter_mut() {
            *p = geometry::index_after_removal(*p, &doomed);
        }

        let mut doomed_functions = self.selected_functions.clone();
        doomed_functions.sort();
        doomed_functions.dedup();
        for i in doomed_functions.into_iter().rev() {
            self.scene.functions.remove(i);
        }

        self.clear_selection();
        self.pending_center = None;
        self.pending_segment = None;
        self.pending_line = None;
    }

    fn new_scene(&mut self) {
        self.push_undo();
        self.scene = Scene::default();
        self.clear_selection();
        self.clear_pending();
        self.helper_text = None;
    }

    fn load_scene(&mut self, scene: Scene) {
        self.push_undo();
        self.scene = scene;
        self.clear_selection();
        self.clear_pending();
    }

    fn add_regular_polygon(&mut self, a: usize, b: usize, sides: usize) {
        self.push_undo();
        let vertices = geometry::regular_polygon(self.scene.points[a], self.scene.points[b], sides);
        let first_new = self.scene.points.len();
        self.scene.points.extend(&vertices[2..]);
        let mut polygon = vec![a, b];
        polygon.extend(first_new..self.scene.points.len());
        self.scene.polygons.push(polygon);
    }

    fn add_function(&mut self, text: String) {
        match expr::parse(&text) {
            Ok(parsed) => {
                self.push_undo();
                self.scene.functions.push((text, parsed));
                self.helper_text = None;
            }
            Err(err) => self.helper_text = Some(format!("f(x) : {err}")),
        }
    }

    fn to_screen_point(&self, p: Point) -> (i32, i32) {
        self.view.to_screen(p.x, p.y)
    }

    // Point and direction of an existing line or segment
    fn line_dir(&self, r: LineRef) -> (Point, (f64, f64)) {
        let (a, b) = match r {
            LineRef::Segment(i) => self.scene.segments[i],
            LineRef::Line(i) => self.scene.lines[i],
        };
        let (pa, pb) = (self.scene.points[a], self.scene.points[b]);
        (pa, (pb.x - pa.x, pb.y - pa.y))
    }

    /// Direction of the line to be placed from reference depending on the active tool
    fn new_line_direction(&self, reference: LineRef) -> (f64, f64) {
        let (_, (dx, dy)) = self.line_dir(reference);
        match self.line_tool {
            LineTool::Parallel => (dx, dy),
            LineTool::Perpendicular => (-dy, dx),
        }
    }

    fn angle_view(&self, angle: &AngleKind) -> Option<AngleView> {
        let (vertex, first, second, degrees) = match *angle {
            AngleKind::ThreePoints(p0, p1, p2) => {
                let (v, a, c) = (self.scene.points[p1], self.scene.points[p0], self.scene.points[p2]);
                let degrees = geometry::angle_between((a.x - v.x, a.y - v.y), (c.x - v.x, c.y - v.y));
                (v, a, c, degrees)
            }
            AngleKind::TwoLines(l1, l2) => {
                let (a1, d1) = self.line_dir(l1);
                let (a2, d2) = self.line_dir(l2);
                let ip = geometry::line_intersection(a1, d1, a2, d2)?;
                let first = Point { x: ip.x + d1.0, y: ip.y + d1.1 };
                let second = Point { x: ip.x + d2.0, y: ip.y + d2.1 };
                (ip, first, second, geometry::angle_between(d1, d2))
            }
        };
        let (vx, vy) = self.to_screen_point(vertex);
        let (x1, y1) = self.to_screen_point(first);
        let (x2, y2) = self.to_screen_point(second);
        Some(AngleView {
            vx,
            vy,
            from: ((y1 - vy) as f64).atan2((x1 - vx) as f64),
            to: ((y2 - vy) as f64).atan2((x2 - vx) as f64),
            degrees,
        })
    }

    fn point_near(&self, wx: f64, wy: f64) -> Option<usize> {
        let target = Point { x: wx, y: wy };
        let tolerance = 15.0 / self.view.scale;
        self.scene.points.iter().position(|&p| distance(p, target) < tolerance)
    }

    fn circle_near(&self, wx: f64, wy: f64) -> Option<usize> {
        let target = Point { x: wx, y: wy };
        let tolerance = 10.0 / self.view.scale;
        self.scene.circles.iter().position(|c| {
            let d = distance(self.scene.points[c.center()], target);
            (d - c.radius(&self.scene.points)).abs() < tolerance
        })
    }

    fn function_near(&self, wx: f64, wy: f64) -> Option<usize> {
        let tolerance = 10.0 / self.view.scale;
        self.scene.functions.iter().position(|(_, expr)| {
            let y = expr.eval(wx);
            y.is_finite() && (y - wy).abs() < tolerance
        })
    }

    fn line_near(&self, click: Point) -> Option<LineRef> {
        let tolerance = 10.0 / self.view.scale;
        let points = &self.scene.points;
        let segment = self.scene.segments.iter().position(|&(a, b)| {
            geometry::point_segment_distance(click, points[a], points[b]) < tolerance
        });
        if let Some(i) = segment {
            return Some(LineRef::Segment(i));
        }
        self.scene
            .lines
            .iter()
            .position(|&(a, b)| {
                let (pa, pb) = (points[a], points[b]);
                geometry::point_line_distance(click, pa, pb.x - pa.x, pb.y - pa.y) < tolerance
            })
            .map(LineRef::Line)
    }

    /// Existing point under the cursor otherwise a new point (without saving undo)
    fn snap_point(&mut self, wx: f64, wy: f64) -> usize {
        if let Some(i) = self.point_near(wx, wy) {
            return i;
        }
        self.scene.points.push(Point { x: wx, y: wy });
        self.scene.points.len() - 1
    }

    fn snap_point_undoable(&mut self, wx: f64, wy: f64) -> usize {
        if let Some(i) = self.point_near(wx, wy) {
            return i;
        }
        self.push_undo();
        self.snap_point(wx, wy)
    }

    fn erase_strokes_near(&mut self, wx: f64, wy: f64) {
        let tolerance = 15.0 / self.view.scale;
        self.scene.strokes.retain(|(_, pts)| !stroke_near(pts, wx, wy, tolerance));
    }


    fn click(&mut self, x: i32, y: i32) -> Option<Prompt> {
        if y < TOOLBAR_HEIGHT {
            return self.click_toolbar(x, y);
        }

        let item_count = self.mode.submenu_labels().len() as i32;
        if let Some(idx) = ui::submenu_hit(x, y, self.mode.menu_x(), item_count) {
            if self.mode == Mode::File {
                return self.file_menu(idx);
            }
            self.select_tool(idx);
            return None;
        }
        if self.mode == Mode::Pen && self.pen_tool == PenTool::Draw {
            if let Some(idx) = ui::pen_palette_hit(x, y, PEN_PALETTE_X) {
                self.pen_color = PEN_COLORS[idx];
                return None;
            }
        }

        if self.show_grid && ui::slider_hit(x, y, self.view.width, self.view.height) {
            self.show_graduations = !self.show_graduations;
            return None;
        }

        self.click_canvas(x, y)
    }

    fn click_toolbar(&mut self, x: i32, y: i32) -> Option<Prompt> {
        let mut prompt = None;
        match ui::toolbar_click(x, y, self.view.width) {
            ui::ToolbarClick::Button(i) => {
                self.mode = Mode::ALL[i];
                if self.mode == Mode::Function {
                    prompt = Some(Prompt::Function);
                }
            }
            ui::ToolbarClick::GridToggle => self.show_grid = !self.show_grid,
            ui::ToolbarClick::Theme => self.theme = self.theme.toggled(),
            ui::ToolbarClick::None => {}
        }
        self.clear_pending();
        prompt
    }

    fn file_menu(&mut self, idx: i32) -> Option<Prompt> {
        self.mode = Mode::Cursor;
        match idx {
            0 => {
                self.new_scene();
                None
            }
            1 => Some(Prompt::Save),
            _ => Some(Prompt::Load),
        }
    }

    fn select_tool(&mut self, idx: i32) {
        match self.mode {
            Mode::Point => {
                self.point_tool = match idx {
                    0 => PointTool::Place,
                    1 => PointTool::Segment,
                    2 => PointTool::Line,
                    _ => PointTool::Move,
                };
                self.pending_segment = None;
                self.pending_line = None;
                self.moving_point = None;
            }
            Mode::Circle => {
                self.circle_tool = if idx == 0 { CircleTool::Basic } else { CircleTool::Radius };
                self.pending_center = None;
            }
            Mode::Polygon => {
                self.polygon_tool = if idx == 0 { PolygonTool::Freeform } else { PolygonTool::Regular };
                self.pending_regular = None;
                self.polygon_in_progress.clear();
            }
            Mode::Line => {
                self.line_tool = if idx == 0 { LineTool::Parallel } else { LineTool::Perpendicular };
                self.pending_reference = None;
                self.line_preview = None;
            }
            Mode::Helpers => {
                self.helper_tool = if idx == 0 { HelperTool::Distance } else { HelperTool::Area };
                self.pending_distance_point = None;
            }
            Mode::Angle => {
                self.angle_tool = if idx == 0 { AngleTool::ThreePoints } else { AngleTool::TwoLines };
                self.pending_angle_points.clear();
                self.pending_angle_line = None;
            }
            Mode::Pen => {
                self.pen_tool = if idx == 0 { PenTool::Draw } else { PenTool::Erase };
            }
            Mode::File | Mode::Cursor | Mode::Function => {}
        }
    }

    fn click_canvas(&mut self, x: i32, y: i32) -> Option<Prompt> {
        let (wx, wy) = self.view.to_world(x, y);
        match self.mode {
            Mode::File | Mode::Function => {}
            Mode::Cursor => {
                self.dragging = true;
                self.drag_start_x = x;
                self.drag_start_y = y;
                self.drag_start_offset_x = self.view.offset_x;
                self.drag_start_offset_y = self.view.offset_y;
            }
            Mode::Point => self.click_point_mode(wx, wy),
            Mode::Circle => return self.click_circle_mode(wx, wy),
            Mode::Polygon => return self.click_polygon_mode(wx, wy),
            Mode::Line => self.click_line_mode(wx, wy),
            Mode::Helpers => self.click_helpers_mode(wx, wy),
            Mode::Angle => self.click_angle_mode(wx, wy),
            Mode::Pen => {
                self.push_undo();
                match self.pen_tool {
                    PenTool::Draw => {
                        self.current_stroke.clear();
                        self.current_stroke.push((wx, wy));
                    }
                    PenTool::Erase => self.erase_strokes_near(wx, wy),
                }
                self.pen_drawing = true;
            }
        }
        None
    }

    fn click_point_mode(&mut self, wx: f64, wy: f64) {
        match self.point_tool {
            PointTool::Place => {
                if let Some(i) = self.point_near(wx, wy) {
                    toggle(&mut self.selected_points, i);
                } else if let Some(i) = self.circle_near(wx, wy) {
                    toggle(&mut self.selected_circles, i);
                } else if let Some(i) = self.function_near(wx, wy) {
                    toggle(&mut self.selected_functions, i);
                } else {
                    self.push_undo();
                    self.scene.points.push(Point { x: wx, y: wy });
                }
            }
            PointTool::Segment => {
                self.push_undo();
                let i = self.snap_point(wx, wy);
                if let Some(segment) = two_clicks(&mut self.pending_segment, i) {
                    self.scene.segments.push(segment);
                }
            }
            PointTool::Line => {
                self.push_undo();
                let i = self.snap_point(wx, wy);
                if let Some(line) = two_clicks(&mut self.pending_line, i) {
                    self.scene.lines.push(line);
                }
            }
            PointTool::Move => {
                if let Some(i) = self.point_near(wx, wy) {
                    self.push_undo();
                    self.moving_point = Some(i);
                }
            }
        }
    }

    fn click_circle_mode(&mut self, wx: f64, wy: f64) -> Option<Prompt> {
        match self.circle_tool {
            CircleTool::Basic => {
                self.push_undo();
                let i = self.snap_point(wx, wy);
                if let Some((center, edge)) = two_clicks(&mut self.pending_center, i) {
                    self.scene.circles.push(Circle::TwoPoint(center, edge));
                }
                None
            }
            CircleTool::Radius => {
                let center = self.snap_point_undoable(wx, wy);
                Some(Prompt::Radius(center))
            }
        }
    }

    fn click_polygon_mode(&mut self, wx: f64, wy: f64) -> Option<Prompt> {
        match self.polygon_tool {
            PolygonTool::Freeform => {
                let clicked = self.point_near(wx, wy);
                let closes = self.polygon_in_progress.len() >= 3
                    && clicked == self.polygon_in_progress.first().copied();
                if closes {
                    self.push_undo();
                    let polygon = std::mem::take(&mut self.polygon_in_progress);
                    self.scene.polygons.push(polygon);
                } else {
                    let i = self.snap_point_undoable(wx, wy);
                    self.polygon_in_progress.push(i);
                }
                None
            }
            PolygonTool::Regular => {
                let i = self.snap_point_undoable(wx, wy);
                two_clicks(&mut self.pending_regular, i).map(|(a, b)| Prompt::Sides(a, b))
            }
        }
    }

    fn click_line_mode(&mut self, wx: f64, wy: f64) {
        let click = Point { x: wx, y: wy };
        match self.pending_reference.take() {
            None => self.pending_reference = self.line_near(click),
            Some(reference) => {
                let (dx, dy) = self.new_line_direction(reference);
                self.push_undo();
                self.scene.points.push(click);
                let anchor = self.scene.points.len() - 1;
                self.scene.directed_lines.push(DirectedLine { anchor, dx, dy });
                self.line_preview = None;
            }
        }
    }

    fn click_helpers_mode(&mut self, wx: f64, wy: f64) {
        match self.helper_tool {
            HelperTool::Distance => {
                let Some(i) = self.point_near(wx, wy) else { return };
                if let Some((a, b)) = two_clicks(&mut self.pending_distance_point, i) {
                    let d = distance(self.scene.points[a], self.scene.points[b]);
                    self.helper_text = Some(format!("Distance: {:.2}", d));
                }
            }
            HelperTool::Area => {
                let click = Point { x: wx, y: wy };
                for poly in &self.scene.polygons {
                    let pts: Vec<Point> = poly.iter().map(|&i| self.scene.points[i]).collect();
                    if geometry::point_in_polygon(click, &pts) {
                        let area = geometry::polygon_area(&pts);
                        self.helper_text = Some(format!("Area: {:.2}", area));
                        break;
                    }
                }
            }
        }
    }

    fn click_angle_mode(&mut self, wx: f64, wy: f64) {
        match self.angle_tool {
            AngleTool::ThreePoints => {
                let Some(i) = self.point_near(wx, wy) else { return };
                self.pending_angle_points.push(i);
                if let [p0, p1, p2] = self.pending_angle_points[..] {
                    self.push_undo();
                    self.scene.angles.push(AngleKind::ThreePoints(p0, p1, p2));
                    self.pending_angle_points.clear();
                }
            }
            AngleTool::TwoLines => {
                let Some(r) = self.line_near(Point { x: wx, y: wy }) else { return };
                match self.pending_angle_line.take() {
                    None => self.pending_angle_line = Some(r),
                    Some(first) => {
                        self.push_undo();
                        self.scene.angles.push(AngleKind::TwoLines(first, r));
                    }
                }
            }
        }
    }

    /// Returns true if the screen needs to be redrawn
    fn mouse_move(&mut self, x: i32, y: i32) -> bool {
        if self.dragging {
            self.view.offset_x = self.drag_start_offset_x + (x - self.drag_start_x) as f64;
            self.view.offset_y = self.drag_start_offset_y - (y - self.drag_start_y) as f64;
        } else if self.mode == Mode::Line && self.pending_reference.is_some() {
            self.line_preview = Some(self.view.to_world(x, y));
        } else if let Some(idx) = self.moving_point {
            let (wx, wy) = self.view.to_world(x, y);
            self.scene.points[idx] = Point { x: wx, y: wy };
        } else if self.mode == Mode::Pen && self.pen_drawing {
            let (wx, wy) = self.view.to_world(x, y);
            match self.pen_tool {
                PenTool::Draw => self.current_stroke.push((wx, wy)),
                PenTool::Erase => self.erase_strokes_near(wx, wy),
            }
        } else {
            return false;
        }
        true
    }

    fn mouse_up(&mut self) -> bool {
        self.dragging = false;
        self.moving_point = None;
        if !self.pen_drawing {
            return false;
        }
        let stroke = std::mem::take(&mut self.current_stroke);
        if self.pen_tool == PenTool::Draw && stroke.len() >= 2 {
            self.scene.strokes.push((self.pen_color, stroke));
        }
        self.pen_drawing = false;
        true
    }


    pub fn redraw(&mut self) {
        let pal = palette(self.theme);
        let view = self.view;
        let (w, h) = (view.width, view.height);

        self.buffer.fill(pal.background);

        if self.show_grid {
            let step = grid_step(view.scale);
            draw_grid_lines(&mut self.buffer, &view, TOOLBAR_HEIGHT, step, pal.grid);
            draw_axes(&mut self.buffer, &view, TOOLBAR_HEIGHT, pal.axis);
            if self.show_graduations {
                draw_graduations(&mut self.buffer, &view, TOOLBAR_HEIGHT, step, pal.axis);
            }
        }

        for polygon in &self.scene.polygons {
            let corners: Vec<(i32, i32)> = polygon
                .iter()
                .map(|&i| view.to_screen(self.scene.points[i].x, self.scene.points[i].y))
                .collect();
            draw_polygon_fill(&mut self.buffer, w, h, &corners, pal.polygon_fill);
            for i in 0..corners.len() {
                let (x0, y0) = corners[i];
                let (x1, y1) = corners[(i + 1) % corners.len()];
                draw_line(&mut self.buffer, w, h, x0, y0, x1, y1, pal.line);
            }
        }

        for pair in self.polygon_in_progress.windows(2) {
            draw_segment(&mut self.buffer, &view, self.scene.points[pair[0]], self.scene.points[pair[1]], pal.line);
        }

        for &(a, b) in &self.scene.segments {
            draw_segment(&mut self.buffer, &view, self.scene.points[a], self.scene.points[b], pal.line);
        }

        for &(a, b) in &self.scene.lines {
            let (pa, pb) = (self.scene.points[a], self.scene.points[b]);
            draw_infinite_line(&mut self.buffer, &view, pa.x, pa.y, pb.x, pb.y, pal.line);
        }

        for dl in &self.scene.directed_lines {
            let a = self.scene.points[dl.anchor];
            draw_infinite_line(&mut self.buffer, &view, a.x, a.y, a.x + dl.dx, a.y + dl.dy, pal.line);
        }

        for (i, (_, expr)) in self.scene.functions.iter().enumerate() {
            let color = if self.selected_functions.contains(&i) { pal.selected_function } else { pal.function };
            draw_function(&mut self.buffer, &view, expr, color);
        }

        if self.mode == Mode::Line {
            if let (Some(reference), Some((px, py))) = (self.pending_reference, self.line_preview) {
                let (dx, dy) = self.new_line_direction(reference);
                let preview_color = blend(pal.line, pal.background, 0.35);
                draw_infinite_line(&mut self.buffer, &view, px, py, px + dx, py + dy, preview_color);
            }
        }

        for angle in &self.scene.angles {
            if let Some(a) = self.angle_view(angle) {
                draw_angle_arc(&mut self.buffer, w, h, a.vx, a.vy, a.from, a.to, 24, pal.angle);
            }
        }

        for (i, c) in self.scene.circles.iter().enumerate() {
            let center = self.scene.points[c.center()];
            let radius = (c.radius(&self.scene.points) * view.scale).round() as i32;
            let (cx, cy) = view.to_screen(center.x, center.y);
            let color = if self.selected_circles.contains(&i) { pal.selected_circle } else { pal.circle };
            draw_circle(&mut self.buffer, w, h, cx, cy, radius, color);
        }

        for (i, p) in self.scene.points.iter().enumerate() {
            let (sx, sy) = view.to_screen(p.x, p.y);
            let color = if self.selected_points.contains(&i) { pal.selected_point } else { pal.point };
            draw_point(&mut self.buffer, w, h, sx, sy, color);
        }

        for (color, pts) in &self.scene.strokes {
            draw_stroke(&mut self.buffer, &view, pts, *color);
        }
        draw_stroke(&mut self.buffer, &view, &self.current_stroke, self.pen_color);

        draw_toolbar(&mut self.buffer, w, h, self.mode as u8, self.show_grid, &pal);

        let labels = self.mode.submenu_labels();
        if !labels.is_empty() {
            let active = match self.mode {
                Mode::Point => self.point_tool as i32,
                Mode::Circle => self.circle_tool as i32,
                Mode::Polygon => self.polygon_tool as i32,
                Mode::Line => self.line_tool as i32,
                Mode::Helpers => self.helper_tool as i32,
                Mode::Angle => self.angle_tool as i32,
                Mode::Pen => self.pen_tool as i32,
                Mode::File | Mode::Cursor | Mode::Function => -1,
            };
            draw_submenu(&mut self.buffer, w, h, self.mode.menu_x(), labels.len() as i32, active, &pal);
        }
        if self.mode == Mode::Pen && self.pen_tool == PenTool::Draw {
            let selected = PEN_COLORS.iter().position(|&c| c == self.pen_color).unwrap_or(0);
            draw_pen_palette(&mut self.buffer, w, h, PEN_PALETTE_X, selected, &pal);
        }

        if self.show_grid {
            draw_slider(&mut self.buffer, w, h, self.show_graduations, &pal);
        }
    }


    fn paint(&self, hdc: HDC) {
        let pal = palette(self.theme);
        let (w, h) = (self.view.width, self.view.height);
        let bmi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 32,
                biCompression: BI_RGB,
                biSizeImage: 0,
                biXPelsPerMeter: 0,
                biYPelsPerMeter: 0,
                biClrUsed: 0,
                biClrImportant: 0,
            },
            bmiColors: [0],
        };
        unsafe {
            StretchDIBits(
                hdc,
                0, 0, w, h,
                0, 0, w, h,
                self.buffer.as_ptr() as *const _,
                &bmi,
                DIB_RGB_COLORS,
                SRCCOPY,
            );
            SetBkMode(hdc, TRANSPARENT);
            SetTextColor(hdc, pal.text);
        }

        self.draw_menu_labels(hdc);
        self.draw_grid_labels(hdc);
        self.draw_point_labels(hdc);
        self.draw_angle_labels(hdc, &pal);
        self.draw_readouts(hdc);
    }

    fn draw_menu_labels(&self, hdc: HDC) {
        let (tx, ty) = ui::theme_text_pos(self.view.width);
        let theme_label = match self.theme {
            Theme::Dark => "Dark theme",
            Theme::Light => "Bright theme",
        };
        text_out(hdc, tx, ty, theme_label);

        let (gx, gy) = ui::grid_label_pos();
        text_out(hdc, gx, gy, "Grid");

        for (i, label) in self.mode.submenu_labels().iter().enumerate() {
            let (lx, ly) = ui::submenu_label_pos(self.mode.menu_x(), i as i32);
            text_out(hdc, lx, ly, label);
        }

        let (fx, fy) = (Mode::Function.menu_x() + 6, TOOLBAR_HEIGHT / 2 - 8);
        text_out(hdc, fx, fy, "f(x)");
    }

    fn submenu_rect(&self) -> Option<(i32, i32, i32, i32)> {
        let items = self.mode.submenu_labels().len() as i32;
        if items == 0 {
            return None;
        }
        let x0 = self.mode.menu_x();
        if self.mode == Mode::Pen {
            let palette_width = PEN_COLORS.len() as i32 * SWATCH_SIZE;
            let x1 = x0 + SUBMENU_WIDTH + palette_width;
            return Some((x0, TOOLBAR_HEIGHT, x1, TOOLBAR_HEIGHT + SUBMENU_ITEM_HEIGHT));
        }
        Some((x0, TOOLBAR_HEIGHT, x0 + SUBMENU_WIDTH, TOOLBAR_HEIGHT + items * SUBMENU_ITEM_HEIGHT))
    }

    fn draw_grid_labels(&self, hdc: HDC) {
        if !self.show_grid {
            return;
        }
        let view = &self.view;
        let (sx, sy) = ui::slider_label_pos(view.width, view.height);
        text_out(hdc, sx, sy, "Tick marks");
        if !self.show_graduations {
            return;
        }

        let step = grid_step(view.scale);
        let decimals: usize = if step < 1.0 { 2 } else if step < 10.0 { 1 } else { 0 };
        let covered = self.submenu_rect();
        let hidden = |px: i32, py: i32| {
            covered.is_some_and(|(x0, y0, x1, y1)| px >= x0 && px < x1 && py >= y0 && py < y1)
        };

        let (ox, oy) = view.to_screen(0.0, 0.0);
        let (columns, rows) = view.visible_cells(TOOLBAR_HEIGHT, step);

        if oy >= TOOLBAR_HEIGHT && oy < view.height {
            for i in columns {
                if i == 0 {
                    continue;
                }
                let wx = i as f64 * step;
                let (sx, _) = view.to_screen(wx, 0.0);
                if sx >= 0 && sx < view.width && !hidden(sx + 4, oy + 4) {
                    text_out(hdc, sx + 4, oy + 4, &format!("{:.*}", decimals, wx));
                }
            }
        }

        if ox >= 0 && ox < view.width {
            for i in rows {
                if i == 0 {
                    continue;
                }
                let wy = i as f64 * step;
                let (_, sy) = view.to_screen(0.0, wy);
                if sy >= TOOLBAR_HEIGHT && sy < view.height && !hidden(ox + 4, sy + 4) {
                    text_out(hdc, ox + 4, sy + 4, &format!("{:.*}", decimals, wy));
                }
            }
        }
    }

    fn draw_point_labels(&self, hdc: HDC) {
        for (i, p) in self.scene.points.iter().enumerate() {
            let (sx, sy) = self.view.to_screen(p.x, p.y);
            let (letter, subscript) = geometry::point_label(i);
            let (lx, ly) = (sx + 6, sy - 16);
            let letter_width = text_width(hdc, &letter);
            text_out(hdc, lx, ly, &letter);
            if let Some(num) = subscript {
                subscript_out(hdc, lx + letter_width, ly + 6, &num);
            }
        }
    }

    fn draw_angle_labels(&self, hdc: HDC, pal: &Palette) {
        if self.scene.angles.is_empty() {
            return;
        }
        unsafe { SetTextColor(hdc, pal.angle) };
        for angle in &self.scene.angles {
            let Some(a) = self.angle_view(angle) else { continue };
            let bisector = a.from + angle_diff(a.from, a.to) / 2.0;
            let label_radius = 34.0;
            let lx = a.vx + (label_radius * bisector.cos()) as i32 - 12;
            let ly = a.vy + (label_radius * bisector.sin()) as i32 - 8;
            text_out(hdc, lx, ly, &format!("{:.1}\u{00B0}", a.degrees));
        }
        unsafe { SetTextColor(hdc, pal.text) };
    }

    fn draw_readouts(&self, hdc: HDC) {
        let mut y = TOOLBAR_HEIGHT + 30;
        for &i in &self.selected_points {
            let p = self.scene.points[i];
            let text = format!("{}: ({:.2}, {:.2})", geometry::point_name(i), p.x, p.y);
            text_out(hdc, self.view.width - 170, y, &text);
            y += 20;
        }

        if let Some(text) = &self.helper_text {
            text_out(hdc, 10, self.view.height - 30, text);
        }

        if let Some(text) = self.hint() {
            text_out(hdc, self.view.width - 280, TOOLBAR_HEIGHT + 6, &text);
        }
    }

    fn hint(&self) -> Option<String> {
        let text = match self.mode {
            Mode::Circle if self.circle_tool == CircleTool::Basic && self.pending_center.is_some() => {
                "click the edge point"
            }
            Mode::Point => match self.point_tool {
                PointTool::Segment if self.pending_segment.is_some() => "click the second point",
                PointTool::Line if self.pending_line.is_some() => "click the second point",
                PointTool::Move if self.moving_point.is_none() => "click and drag a point",
                _ => return None,
            },
            Mode::Polygon => match self.polygon_tool {
                PolygonTool::Freeform if !self.polygon_in_progress.is_empty() => {
                    "click the first point to close"
                }
                PolygonTool::Regular if self.pending_regular.is_some() => "click the second point",
                _ => return None,
            },
            Mode::Line if self.pending_reference.is_none() => "click an existing line or segment",
            Mode::Line => "move and click to place the line",
            Mode::Helpers
                if self.helper_tool == HelperTool::Distance
                    && self.pending_distance_point.is_some() =>
            {
                "click the second point"
            }
            Mode::Angle => match self.angle_tool {
                AngleTool::ThreePoints if !self.pending_angle_points.is_empty() => {
                    return Some(format!("click point {} of 3", self.pending_angle_points.len() + 1));
                }
                AngleTool::TwoLines if self.pending_angle_line.is_some() => "click the second line",
                _ => return None,
            },
            _ => return None,
        };
        Some(text.to_string())
    }
}

fn draw_segment(buffer: &mut [u32], view: &View, a: Point, b: Point, color: u32) {
    let (x0, y0) = view.to_screen(a.x, a.y);
    let (x1, y1) = view.to_screen(b.x, b.y);
    draw_line(buffer, view.width, view.height, x0, y0, x1, y1, color);
}

fn draw_stroke(buffer: &mut [u32], view: &View, points: &[(f64, f64)], color: u32) {
    for pair in points.windows(2) {
        let (x0, y0) = view.to_screen(pair[0].0, pair[0].1);
        let (x1, y1) = view.to_screen(pair[1].0, pair[1].1);
        draw_thick_line(buffer, view.width, view.height, x0, y0, x1, y1, 1, color);
    }
}

fn draw_function(buffer: &mut [u32], view: &View, expr: &expr::Expr, color: u32) {
    let mut prev: Option<(i32, i32)> = None;
    for sx in 0..view.width {
        let (wx, _) = view.to_world(sx, 0);
        let wy = expr.eval(wx);
        let point = wy.is_finite().then(|| view.to_screen(wx, wy));
        if let (Some((x0, y0)), Some((x1, y1))) = (prev, point) {
            if (y1 - y0).unsigned_abs() < view.height as u32 * 4 {
                draw_line(buffer, view.width, view.height, x0, y0, x1, y1, color);
            }
        }
        prev = point;
    }
}

fn toggle(list: &mut Vec<usize>, item: usize) {
    if list.contains(&item) {
        list.retain(|&s| s != item);
    } else {
        list.push(item);
    }
}

fn two_clicks(pending: &mut Option<usize>, index: usize) -> Option<(usize, usize)> {
    match pending.take() {
        None => {
            *pending = Some(index);
            None
        }
        Some(start) => (start != index).then_some((start, index)),
    }
}

fn stroke_near(stroke: &[(f64, f64)], wx: f64, wy: f64, tol: f64) -> bool {
    stroke.iter().any(|&(x, y)| {
        let dx = x - wx;
        let dy = y - wy;
        (dx * dx + dy * dy).sqrt() < tol
    })
}

fn text_out(hdc: HDC, x: i32, y: i32, text: &str) {
    let wide = to_wide(text);
    unsafe {
        TextOutW(hdc, x, y, wide.as_ptr(), (wide.len() - 1) as i32);
    }
}

fn text_width(hdc: HDC, text: &str) -> i32 {
    let wide = to_wide(text);
    let mut size = SIZE { cx: 0, cy: 0 };
    unsafe {
        GetTextExtentPoint32W(hdc, wide.as_ptr(), (wide.len() - 1) as i32, &mut size);
    }
    size.cx
}

fn subscript_out(hdc: HDC, x: i32, y: i32, text: &str) {
    let face = to_wide("");
    unsafe {
        let small_font = CreateFontW(11, 0, 0, 0, 400, 0, 0, 0, 0, 0, 0, 0, 0, face.as_ptr());
        let old_font = SelectObject(hdc, small_font);
        text_out(hdc, x, y, text);
        SelectObject(hdc, old_font);
        DeleteObject(small_font);
    }
}


static mut APP: Option<AppState> = None;

fn app_state() -> Option<&'static mut AppState> {
    unsafe { (*(&raw mut APP)).as_mut() }
}

fn repaint(hwnd: HWND, app: &mut AppState) {
    app.redraw();
    unsafe {
        InvalidateRect(hwnd, std::ptr::null(), 0);
    }
}

fn cursor_pos(lparam: LPARAM) -> (i32, i32) {
    ((lparam & 0xffff) as i16 as i32, ((lparam >> 16) & 0xffff) as i16 as i32)
}

fn on_left_down(hwnd: HWND, lparam: LPARAM) {
    let (x, y) = cursor_pos(lparam);
    let Some(app) = app_state() else { return };
    let prompt = app.click(x, y);
    let pal = palette(app.theme);
    repaint(hwnd, app);

    if let Some(prompt) = prompt {
        run_prompt(hwnd, prompt, pal);
    }
}

fn run_prompt(hwnd: HWND, prompt: Prompt, pal: Palette) {
    match prompt {
        Prompt::Radius(center) => {
            let Some(r) = dialog::ask_value(hwnd, "Radius", pal.toolbar_bg, pal.text) else { return };
            if r <= 0.0 {
                return;
            }
            if let Some(app) = app_state() {
                app.push_undo();
                app.scene.circles.push(Circle::Radius(center, r));
                repaint(hwnd, app);
            }
        }
        Prompt::Sides(a, b) => {
            let Some(v) = dialog::ask_value(hwnd, "Sides", pal.toolbar_bg, pal.text) else { return };
            let sides = v.round() as i64;
            if sides < 3 {
                return;
            }
            if let Some(app) = app_state() {
                app.add_regular_polygon(a, b, sides as usize);
                repaint(hwnd, app);
            }
        }
        Prompt::Save => {
            let Some(path) = storage::pick_save_path(hwnd) else { return };
            if let Some(app) = app_state() {
                storage::write_scene(&path, &app.scene);
            }
        }
        Prompt::Load => {
            let Some(path) = storage::pick_open_path(hwnd) else { return };
            let Some(scene) = storage::read_scene(&path) else { return };
            if let Some(app) = app_state() {
                app.load_scene(scene);
                repaint(hwnd, app);
            }
        }
        Prompt::Function => {
            let Some(text) = dialog::ask_function(hwnd, pal.toolbar_bg, pal.text) else { return };
            if let Some(app) = app_state() {
                app.add_function(text);
                repaint(hwnd, app);
            }
        }
    }
}

fn on_paint(hwnd: HWND) {
    unsafe {
        let mut ps: PAINTSTRUCT = std::mem::zeroed();
        let hdc = BeginPaint(hwnd, &mut ps);
        if let Some(app) = app_state() {
            app.paint(hdc);
        }
        EndPaint(hwnd, &ps);
    }
}

pub extern "system" fn window_proc(hwnd: HWND, msg: UINT, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_DESTROY => {
            unsafe { PostQuitMessage(0) };
            0
        }
        WM_SIZE => {
            let width = (lparam & 0xffff) as i32;
            let height = ((lparam >> 16) & 0xffff) as i32;
            if let Some(app) = app_state() {
                app.resize(width, height);
                app.redraw();
            }
            0
        }
        WM_LBUTTONDOWN => {
            on_left_down(hwnd, lparam);
            0
        }
        WM_LBUTTONUP => {
            if let Some(app) = app_state() {
                if app.mouse_up() {
                    repaint(hwnd, app);
                }
            }
            0
        }
        WM_MOUSEMOVE => {
            let (x, y) = cursor_pos(lparam);
            if let Some(app) = app_state() {
                if app.mouse_move(x, y) {
                    repaint(hwnd, app);
                }
            }
            0
        }
        WM_MOUSEWHEEL => {
            if let Some(app) = app_state() {
                let delta = ((wparam >> 16) & 0xffff) as i16;
                app.view.scale *= if delta > 0 { 1.1 } else { 1.0 / 1.1 };
                repaint(hwnd, app);
            }
            0
        }
        WM_KEYDOWN => {
            if let Some(app) = app_state() {
                match wparam {
                    VK_DELETE => {
                        app.push_undo();
                        app.delete_selected();
                    }
                    KEY_A if control::ctrl_down() => app.select_all_points(),
                    KEY_Z if control::ctrl_down() => app.undo(),
                    _ => return 0,
                }
                repaint(hwnd, app);
            }
            0
        }
        WM_PAINT => {
            on_paint(hwnd);
            0
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

pub fn init(width: i32, height: i32) {
    let mut state = AppState::new(width, height);
    state.redraw();
    unsafe {
        APP = Some(state);
    }
}