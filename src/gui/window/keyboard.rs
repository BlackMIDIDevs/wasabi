use egui::{Color32, Mesh, Pos2, Rect, Sense, Ui};

use crate::midi::MIDIColor;

use super::keyboard_layout::KeyboardView;

pub struct GuiKeyboard {}

impl GuiKeyboard {
    pub fn new() -> GuiKeyboard {
        GuiKeyboard {}
    }

    pub fn draw(&mut self, ui: &mut Ui, key_view: &KeyboardView, colors: &Vec<Option<MIDIColor>>) -> usize {
        let (rect, _) = ui.allocate_exact_size(ui.available_size(), Sense::click());

        let mut mesh = Mesh::default();

        let painter = ui.painter();

        let key_density = (((rect.width() as usize / key_view.visible_range.len()) / 8) as u8).max(1);

        let onepx = painter.round_to_pixel(1.0 * key_density as f32);

        let md_height = rect.height() * 0.042;

        let black_key_overlap = md_height / 1.5;
        let top = rect.top() + md_height;
        let bottom = rect.bottom();
        let black_bottom = rect.bottom() - rect.height() * 0.34;

        let map_x = |num: f32| rect.left() + num * rect.width();

        fn map_color(col: MIDIColor) -> Color32 {
            Color32::from_rgb(col.red(), col.green(), col.blue())
        }

        let mut pressed: usize = 0;

        for (i, key) in key_view.iter_visible_keys() {
            if !key.black {
                let color = colors[i].map(map_color).unwrap_or(Color32::WHITE);

                if color == Color32::WHITE {  // Not pressed
                    let white_key_bottom = md_height;

                    let top_left1 = Pos2::new(map_x(key.left), top);
                    let bottom_right1 = Pos2::new(map_x(key.right), bottom - white_key_bottom);
                    let rect1 = Rect::from_min_max(top_left1, bottom_right1);

                    let top_left2 = Pos2::new(map_x(key.left), bottom - white_key_bottom);
                    let bottom_right2 = Pos2::new(map_x(key.right), bottom);
                    let rect2 = Rect::from_min_max(top_left2, bottom_right2);
                    let color2 = Color32::from_rgb(120, 120, 120);

                    mesh.add_colored_rect(rect1, color);
                    mesh.add_colored_rect(rect2, color2);
                } else {  // Pressed
                    pressed += 1;
                    let top_left = Pos2::new(map_x(key.left), top);
                    let bottom_right = Pos2::new(map_x(key.right), bottom);
                    let rect = Rect::from_min_max(top_left, bottom_right);
                    mesh.add_colored_rect(rect, color);
                }
                let color3 = Color32::from_rgb(20, 20, 20);
                let top_left3 = Pos2::new(map_x(key.right), top);
                let bottom_right3 = Pos2::new(map_x(key.right) - onepx, bottom);
                let rect3 = Rect::from_min_max(top_left3, bottom_right3);
                mesh.add_colored_rect(rect3, color3);
            }
        }

        let bar_top_left = Pos2::new(rect.left(), rect.top());
        let bar_bottom_right = Pos2::new(rect.right(), top);
        let bar_rect = Rect::from_min_max(bar_top_left, bar_bottom_right);
        let bar_color = Color32::from_rgb(65, 0, 30);

        mesh.add_colored_rect(bar_rect, bar_color);

        for (i, key) in key_view.iter_visible_keys() {
            if key.black {
                let color = colors[i].map(map_color).unwrap_or(Color32::BLACK);

                if color == Color32::BLACK {  // Not pressed
                    let black_key_bottom = md_height;

                    let top_left1 = Pos2::new(map_x(key.left) - onepx, top);
                    let bottom_right1 = Pos2::new(map_x(key.right) + onepx, black_bottom);
                    let rect1 = Rect::from_min_max(top_left1, bottom_right1);

                    let top_left2 = Pos2::new(map_x(key.left), top - black_key_overlap);
                    let bottom_right2 = Pos2::new(map_x(key.right), black_bottom - black_key_bottom);
                    let rect2 = Rect::from_min_max(top_left2, bottom_right2);
                    let color2 = Color32::from_rgb(30, 30, 30);

                    mesh.add_colored_rect(rect1, color);
                    mesh.add_colored_rect(rect2, color2);
                } else {  // Pressed
                    pressed += 1;
                    let top_left1 = Pos2::new(map_x(key.left), top);
                    let bottom_right1 = Pos2::new(map_x(key.right), black_bottom);
                    let rect1 = Rect::from_min_max(top_left1, bottom_right1);
                    let color1 = Color32::from_rgb(color.r() / 2, color.g() / 2, color.b() / 2);

                    let top_left2 = Pos2::new(map_x(key.left) + onepx, top);
                    let bottom_right2 = Pos2::new(map_x(key.right) - onepx, black_bottom - onepx);
                    let rect2 = Rect::from_min_max(top_left2, bottom_right2);

                    mesh.add_colored_rect(rect1, color1);
                    mesh.add_colored_rect(rect2, color);
                }
            }
        }

        ui.painter().add(mesh);

        pressed
    }
}
