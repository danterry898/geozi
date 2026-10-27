use crate::app::AppState;
use crate::win32::{GetKeyState, VK_CONTROL};

pub fn ctrl_down() -> bool {
    // bit of the high weight = key pressed so negative value
    unsafe { GetKeyState(VK_CONTROL) < 0 }
}

impl AppState {
    pub fn push_undo(&mut self) {
        self.undo_stack.push(self.scene.clone());
    }

    pub fn undo(&mut self) {
        if let Some(scene) = self.undo_stack.pop() {
            self.scene = scene;
            self.clear_selection();
            self.clear_pending();
        }
    }

    pub fn select_all_points(&mut self) {
        self.selected_points = (0..self.scene.points.len()).collect();
    }
}