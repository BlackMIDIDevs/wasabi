use egui::{Color32, Mesh, Pos2, Rect, Sense, Ui};

use super::keyboard_layout::KeyboardView;

pub struct GuiKeyboard {}

impl GuiKeyboard {
    pub fn new() -> GuiKeyboard {
        GuiKeyboard {}
    }

    pub fn draw(&mut self, ui: &mut Ui, key_view: &KeyboardView) {
        let (rect, _) = ui.allocate_exact_size(ui.available_size(), Sense::click());

        let mut mesh = Mesh::default();

        let top = rect.top();
        let bottom = rect.bottom();
        let black_bottom = rect.bottom() - rect.height() * 0.4;

        let map_x = |num: f32| rect.left() + num * rect.width();

        for (_, key) in key_view.iter_keys() {
            if !key.black {
                let top_left = Pos2::new(map_x(key.left), top);
                let bottom_right = Pos2::new(map_x(key.right), bottom);

                let rect = Rect::from_min_max(top_left, bottom_right);
                let color = Color32::WHITE;

                mesh.add_colored_rect(rect, color)
            }
        }

        for (_, key) in key_view.iter_keys() {
            if key.black {
                let top_left = Pos2::new(map_x(key.left), top);
                let bottom_right = Pos2::new(map_x(key.right), black_bottom);

                let rect = Rect::from_min_max(top_left, bottom_right);
                let color = Color32::BLACK;

                mesh.add_colored_rect(rect, color)
            }
        }

        ui.painter().add(mesh);
    }
}
