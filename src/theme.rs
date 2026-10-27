#[derive(Clone, Copy)]
pub struct Palette {
    pub background: u32,
    pub toolbar_bg: u32,
    pub button_active: u32,
    pub icon: u32,
    pub text: u32,
    pub grid: u32,
    pub axis: u32,
    pub point: u32,
    pub selected_point: u32,
    pub line: u32,
    pub circle: u32,
    pub selected_circle: u32,
    pub polygon_fill: u32,
    pub angle: u32,
    pub function: u32,
    pub selected_function: u32,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Theme {
    Dark,
    Light,
}

impl Theme {
    pub fn toggled(self) -> Theme {
        match self {
            Theme::Dark => Theme::Light,
            Theme::Light => Theme::Dark,
        }
    }
}

pub fn palette(theme: Theme) -> Palette {
    match theme {
        Theme::Dark => Palette {
            background: 0x00202020,
            toolbar_bg: 0x00303030,
            button_active: 0x00555555,
            icon: 0x00ffffff,
            text: 0x00ffffff,
            grid: 0x00383838,
            axis: 0x00606060,
            point: 0x00ffffff,
            selected_point: 0x0000ff00,
            line: 0x00ffcc00,
            circle: 0x0000ccff,
            selected_circle: 0x0000ff88,
            polygon_fill: 0x00335533,
            angle: 0x00ff5fd4,
            function: 0x009d4edd,
            selected_function: 0x00ffffff,
        },
        Theme::Light => Palette {
            background: 0x00f0f0f0,
            toolbar_bg: 0x00d8d8d8,
            button_active: 0x00b0b0b0,
            icon: 0x00202020,
            text: 0x00202020,
            grid: 0x00cccccc,
            axis: 0x00999999,
            point: 0x00202020,
            selected_point: 0x0000aa00,
            line: 0x00cc7a00,
            circle: 0x000077aa,
            selected_circle: 0x00005588,
            polygon_fill: 0x00cfe8cf,
            angle: 0x00aa00aa,
            function: 0x007b2cbf,
            selected_function: 0x00000000,
        },
    }
}