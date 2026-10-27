use crate::render::{draw_circle, draw_line, draw_point, fill_rect, put_pixel};
use crate::theme::Palette;

pub const BUTTON_SIZE: i32 = 40;
pub const BUTTON_COUNT: i32 = 10;
pub const TOOLBAR_HEIGHT: i32 = BUTTON_SIZE;
pub const SUBMENU_WIDTH: i32 = 190;
pub const SUBMENU_ITEM_HEIGHT: i32 = 28;
pub const SLIDER_WIDTH: i32 = 60;
pub const SLIDER_HEIGHT: i32 = 26;
pub const SLIDER_MARGIN: i32 = 14;
pub const THEME_TEXT_ZONE: i32 = 150;
pub const GRID_BOX_SIZE: i32 = 16;
pub const SWATCH_SIZE: i32 = 20;

pub const PEN_COLORS: [u32; 8] = [
    0x00e63946, 0x00f4a261, 0x00e9c46a, 0x002a9d8f,
    0x00457b9d, 0x009b5de5, 0x00ffffff, 0x00000000,
];

pub enum ToolbarClick {
    /// Mode button index from left to right.
    Button(usize),
    GridToggle,
    Theme,
    None,
}

pub fn grid_box_pos() -> (i32, i32) {
    (BUTTON_SIZE * BUTTON_COUNT + 20, TOOLBAR_HEIGHT / 2 - GRID_BOX_SIZE / 2)
}

pub fn grid_label_pos() -> (i32, i32) {
    (BUTTON_SIZE * BUTTON_COUNT + 44, TOOLBAR_HEIGHT / 2 - 8)
}

pub fn theme_text_pos(width: i32) -> (i32, i32) {
    (width - THEME_TEXT_ZONE + 10, TOOLBAR_HEIGHT / 2 - 8)
}

pub fn submenu_label_pos(x0: i32, index: i32) -> (i32, i32) {
    (x0 + 8, TOOLBAR_HEIGHT + index * SUBMENU_ITEM_HEIGHT + 6)
}

pub fn slider_label_pos(width: i32, height: i32) -> (i32, i32) {
    (width - SLIDER_MARGIN - SLIDER_WIDTH - 90, height - SLIDER_MARGIN - SLIDER_HEIGHT + 4)
}

pub fn toolbar_click(x: i32, y: i32, width: i32) -> ToolbarClick {
    if y >= TOOLBAR_HEIGHT {
        return ToolbarClick::None;
    }
    if x >= width - THEME_TEXT_ZONE {
        return ToolbarClick::Theme;
    }
    if x < BUTTON_SIZE * BUTTON_COUNT {
        return ToolbarClick::Button((x.max(0) / BUTTON_SIZE) as usize);
    }
    let (box_x, box_y) = grid_box_pos();
    if x >= box_x && x < box_x + GRID_BOX_SIZE && y >= box_y && y < box_y + GRID_BOX_SIZE {
        return ToolbarClick::GridToggle;
    }
    ToolbarClick::None
}

pub fn submenu_hit(x: i32, y: i32, x0: i32, item_count: i32) -> Option<i32> {
    if x < x0 || x >= x0 + SUBMENU_WIDTH {
        return None;
    }
    if y < TOOLBAR_HEIGHT || y >= TOOLBAR_HEIGHT + item_count * SUBMENU_ITEM_HEIGHT {
        return None;
    }
    Some((y - TOOLBAR_HEIGHT) / SUBMENU_ITEM_HEIGHT)
}

pub fn slider_hit(x: i32, y: i32, width: i32, height: i32) -> bool {
    let x0 = width - SLIDER_MARGIN - SLIDER_WIDTH;
    let y0 = height - SLIDER_MARGIN - SLIDER_HEIGHT;
    x >= x0 && x < x0 + SLIDER_WIDTH && y >= y0 && y < y0 + SLIDER_HEIGHT
}

pub fn pen_palette_hit(x: i32, y: i32, x0: i32) -> Option<usize> {
    if y < TOOLBAR_HEIGHT || y >= TOOLBAR_HEIGHT + SUBMENU_ITEM_HEIGHT {
        return None;
    }
    let rel = x - x0;
    if rel < 0 {
        return None;
    }
    let idx = (rel / SWATCH_SIZE) as usize;
    if idx < PEN_COLORS.len() {
        Some(idx)
    } else {
        None
    }
}

pub fn draw_toolbar(buffer: &mut [u32], width: i32, height: i32, mode: u8, show_grid: bool, palette: &Palette) {
    fill_rect(buffer, width, height, 0, 0, width, TOOLBAR_HEIGHT, palette.toolbar_bg);

    for id in 0..BUTTON_COUNT as u8 {
        let bx = id as i32 * BUTTON_SIZE;
        let bg = if id == mode { palette.button_active } else { palette.toolbar_bg };
        fill_rect(buffer, width, height, bx + 2, 2, bx + BUTTON_SIZE - 2, TOOLBAR_HEIGHT - 2, bg);

        let cx = bx + BUTTON_SIZE / 2;
        let cy = TOOLBAR_HEIGHT / 2;
        let mut line = |x0: i32, y0: i32, x1: i32, y1: i32| {
            draw_line(buffer, width, height, cx + x0, cy + y0, cx + x1, cy + y1, palette.icon);
        };
        match id {
            0 => {
                line(-8, -10, 4, -10);
                line(-8, -10, -8, 10);
                line(-8, 10, 8, 10);
                line(8, 10, 8, -6);
                line(4, -10, 8, -6);
            }
            1 => {
                line(-8, -10, 2, 10);
                line(-8, -10, -8, 3);
                line(-8, -10, 5, -4);
            }
            2 => draw_point(buffer, width, height, cx, cy, palette.icon),
            3 => draw_circle(buffer, width, height, cx, cy, 9, palette.icon),
            4 => {
                line(-9, 7, 9, 7);
                line(-9, 7, 0, -9);
                line(9, 7, 0, -9);
            }
            5 => line(-10, 8, 10, -8),
            6 => {
                line(-10, 0, 10, 0);
                line(-6, -5, -6, 5);
                line(0, -5, 0, 5);
                line(6, -5, 6, 5);
            }
            7 => {
                line(-8, 9, 9, 9);
                line(-8, 9, -8, -9);
                line(-8, 3, -1, 9);
            }
            8 => {
                line(-8, 9, 7, -8);
                line(-8, 9, -4, 9);
                line(-8, 9, -8, 5);
            }
            _ => {}
        }
    }

    let (box_x, box_y) = grid_box_pos();
    for y in box_y..box_y + GRID_BOX_SIZE {
        for x in box_x..box_x + GRID_BOX_SIZE {
            let edge = x == box_x || x == box_x + GRID_BOX_SIZE - 1 || y == box_y || y == box_y + GRID_BOX_SIZE - 1;
            if edge || show_grid {
                put_pixel(buffer, width, height, x, y, palette.icon);
            }
        }
    }
}

pub fn draw_submenu(buffer: &mut [u32], width: i32, height: i32, x0: i32, item_count: i32, active: i32, palette: &Palette) {
    for i in 0..item_count {
        let y0 = TOOLBAR_HEIGHT + i * SUBMENU_ITEM_HEIGHT;
        let bg = if i == active { palette.button_active } else { palette.toolbar_bg };
        fill_rect(buffer, width, height, x0, y0, x0 + SUBMENU_WIDTH, y0 + SUBMENU_ITEM_HEIGHT, bg);
    }

    let x1 = x0 + SUBMENU_WIDTH;
    let bottom = TOOLBAR_HEIGHT + item_count * SUBMENU_ITEM_HEIGHT;
    fill_rect(buffer, width, height, x0, bottom - 1, x1, bottom, palette.icon);
    fill_rect(buffer, width, height, x0, TOOLBAR_HEIGHT, x0 + 1, bottom, palette.icon);
    fill_rect(buffer, width, height, x1 - 1, TOOLBAR_HEIGHT, x1, bottom, palette.icon);
    for i in 1..item_count {
        let y = TOOLBAR_HEIGHT + i * SUBMENU_ITEM_HEIGHT;
        fill_rect(buffer, width, height, x0, y, x1, y + 1, palette.icon);
    }
}

pub fn draw_pen_palette(buffer: &mut [u32], width: i32, height: i32, x0: i32, selected: usize, palette: &Palette) {
    let total_width = SWATCH_SIZE * PEN_COLORS.len() as i32;
    fill_rect(buffer, width, height, x0, TOOLBAR_HEIGHT, x0 + total_width, TOOLBAR_HEIGHT + SUBMENU_ITEM_HEIGHT, palette.toolbar_bg);

    for (i, &color) in PEN_COLORS.iter().enumerate() {
        let sx = x0 + i as i32 * SWATCH_SIZE + 2;
        let sy = TOOLBAR_HEIGHT + 4;
        let (right, bottom) = (sx + SWATCH_SIZE - 4, sy + SWATCH_SIZE - 8);
        fill_rect(buffer, width, height, sx, sy, right, bottom, color);
        if i == selected {
            fill_rect(buffer, width, height, sx - 1, sy - 1, right + 1, sy, palette.icon);
            fill_rect(buffer, width, height, sx - 1, bottom, right + 1, bottom + 1, palette.icon);
        }
    }
}

pub fn draw_slider(buffer: &mut [u32], width: i32, height: i32, on: bool, palette: &Palette) {
    let x0 = width - SLIDER_MARGIN - SLIDER_WIDTH;
    let y0 = height - SLIDER_MARGIN - SLIDER_HEIGHT;
    let track = if on { palette.button_active } else { palette.toolbar_bg };
    fill_rect(buffer, width, height, x0, y0, x0 + SLIDER_WIDTH, y0 + SLIDER_HEIGHT, track);

    let knob_size = SLIDER_HEIGHT - 6;
    let knob_x = if on { x0 + SLIDER_WIDTH - knob_size - 3 } else { x0 + 3 };
    let knob_y = y0 + 3;
    fill_rect(buffer, width, height, knob_x, knob_y, knob_x + knob_size, knob_y + knob_size, palette.icon);
}