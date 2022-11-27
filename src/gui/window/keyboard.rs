use egui::{Color32, Mesh, Pos2, Rect, Sense, Ui};

use crate::midi::MIDIColor;

use super::keyboard_layout::KeyboardView;

enum GradDirection {
    Up,
    Down,
}

pub struct GuiKeyboard {}

impl GuiKeyboard {
    pub fn new() -> GuiKeyboard {
        GuiKeyboard {}
    }

    pub fn draw(
        &mut self,
        ui: &mut Ui,
        key_view: &KeyboardView,
        colors: &[Option<MIDIColor>],
        bar_color: &Color32,
    ) {
        let (rect, _) = ui.allocate_exact_size(ui.available_size(), Sense::click());
        let mut mesh = Mesh::default();
        let key_density = ((rect.width() / key_view.visible_range.len() as f32) / 8.0)
            .max(1.0)
            .min(5.0);
        let onepx = ui.painter().round_to_pixel(key_density);

        let md_height = rect.height() * 0.044;
        let bar = rect.height() * 0.06;

        let black_key_overlap = bar / 2.0;
        let top = rect.top() + bar;
        let bottom = rect.bottom();
        let black_bottom = rect.bottom() - rect.height() * 0.34;
        let map_x = |num: f32| rect.left() + num * rect.width();
        fn map_color(col: MIDIColor) -> Color32 {
            Color32::from_rgb(col.red(), col.green(), col.blue())
        }

        for (i, key) in key_view.iter_visible_keys() {
            if !key.black {
                if let Some(color) = colors[i].map(map_color) {
                    // Pressed
                    // Surface
                    let top_left = Pos2::new(map_x(key.left), top);
                    let bottom_right = Pos2::new(map_x(key.right), bottom);
                    let rect = Rect::from_min_max(top_left, bottom_right);

                    // Bottom line
                    let top_left2 = Pos2::new(map_x(key.left), bottom);
                    let bottom_right2 = Pos2::new(map_x(key.right), bottom - onepx);
                    let rect2 = Rect::from_min_max(top_left2, bottom_right2);
                    let color2 = Color32::from_rgb(color.r() / 2, color.g() / 2, color.b() / 2);

                    draw_gradient_darken(&mut mesh, rect, color, GradDirection::Up, 0.5);
                    mesh.add_colored_rect(rect2, color2);
                } else {
                    // Not pressed
                    let white_key_bottom = md_height;
                    let color = Color32::WHITE;

                    // Surface
                    let top_left1 = Pos2::new(map_x(key.left), top);
                    let bottom_right1 = Pos2::new(map_x(key.right), bottom - white_key_bottom);
                    let rect1 = Rect::from_min_max(top_left1, bottom_right1);

                    // Bottom part
                    let top_left2 = Pos2::new(map_x(key.left), bottom - white_key_bottom);
                    let bottom_right2 = Pos2::new(map_x(key.right), bottom);
                    let rect2 = Rect::from_min_max(top_left2, bottom_right2);
                    let color2 = Color32::from_rgb(190, 190, 190);

                    // Bottom line separator
                    let top_left3 = Pos2::new(map_x(key.left), bottom - white_key_bottom);
                    let bottom_right3 =
                        Pos2::new(map_x(key.right), bottom - white_key_bottom + onepx);
                    let rect3 = Rect::from_min_max(top_left3, bottom_right3);
                    let color3 = Color32::from_rgb(100, 100, 100);

                    draw_gradient_darken(&mut mesh, rect1, color, GradDirection::Up, 0.6);
                    draw_gradient_darken(&mut mesh, rect2, color2, GradDirection::Down, 0.3);
                    mesh.add_colored_rect(rect3, color3);
                }
                // White key borders
                let color4 = Color32::from_rgb(45, 45, 45);
                let top_left4 = Pos2::new(map_x(key.right), top);
                let bottom_right4 = Pos2::new(map_x(key.right) - onepx, bottom);
                let rect4 = Rect::from_min_max(top_left4, bottom_right4);
                mesh.add_colored_rect(rect4, color4);
            }
        }

        let bar_top_left = Pos2::new(rect.left(), rect.top());
        let bar_bottom_right = Pos2::new(rect.right(), top - onepx);
        let bar_rect = Rect::from_min_max(bar_top_left, bar_bottom_right);
        draw_gradient_darken(&mut mesh, bar_rect, *bar_color, GradDirection::Up, 0.3);

        let sepr_top_left = Pos2::new(rect.left(), top - onepx);
        let sepr_bottom_right = Pos2::new(rect.right(), top);
        let sepr_rect = Rect::from_min_max(sepr_top_left, sepr_bottom_right);
        let sepr_color = Color32::from_rgb(60, 60, 60);
        mesh.add_colored_rect(sepr_rect, sepr_color);

        for (i, key) in key_view.iter_visible_keys() {
            if key.black {
                if let Some(color) = colors[i].map(map_color) {
                    // Pressed
                    // Outline
                    let top_left1 = Pos2::new(map_x(key.left), top);
                    let bottom_right1 = Pos2::new(map_x(key.right), black_bottom);
                    let rect1 = Rect::from_min_max(top_left1, bottom_right1);
                    let color1 = Color32::from_rgb(color.r() / 3, color.g() / 3, color.b() / 3);

                    // Surface
                    let top_left2 = Pos2::new(map_x(key.left) + onepx, top - onepx);
                    let bottom_right2 = Pos2::new(map_x(key.right) - onepx, black_bottom - onepx);
                    let rect2 = Rect::from_min_max(top_left2, bottom_right2);

                    mesh.add_colored_rect(rect1, color1);
                    draw_gradient_darken(&mut mesh, rect2, color, GradDirection::Down, 0.55);
                } else {
                    // Not pressed
                    let black_key_bottom = md_height;

                    // Outline + Bottom part
                    let top_left1 = Pos2::new(map_x(key.left) - onepx, top);
                    let bottom_right1 = Pos2::new(map_x(key.right) + onepx, black_bottom);
                    let rect1 = Rect::from_min_max(top_left1, bottom_right1);
                    let color1 = Color32::from_rgb(65, 65, 65);

                    // Surface
                    let top_left2 = Pos2::new(map_x(key.left) + onepx, top - black_key_overlap);
                    let bottom_right2 =
                        Pos2::new(map_x(key.right) - onepx, black_bottom - black_key_bottom);
                    let rect2 = Rect::from_min_max(top_left2, bottom_right2);
                    let color2 = Color32::from_rgb(38, 38, 38);

                    draw_gradient_darken(&mut mesh, rect1, color1, GradDirection::Up, 0.55);
                    draw_gradient_darken(&mut mesh, rect2, color2, GradDirection::Down, 0.3);
                }
            }
        }

        ui.painter().add(mesh);
    }
}

fn draw_gradient_darken(mesh: &mut Mesh, rect: Rect, color: Color32, direction: GradDirection, mult: f32) {
    let idx = mesh.vertices.len() as u32;
    mesh.add_triangle(idx + 0, idx + 1, idx + 2);
    mesh.add_triangle(idx + 2, idx + 1, idx + 3);
    let darken = Color32::from_rgb(
        (color.r() as f32 * mult) as u8,
        (color.g() as f32 * mult) as u8,
        (color.b() as f32 * mult) as u8,
    );
    match direction{
        GradDirection::Up => {
            mesh.colored_vertex(rect.left_top(), darken);
            mesh.colored_vertex(rect.right_top(), darken);
            mesh.colored_vertex(rect.left_bottom(), color);
            mesh.colored_vertex(rect.right_bottom(), color);
        }
        GradDirection::Down => {
            mesh.colored_vertex(rect.left_top(), color);
            mesh.colored_vertex(rect.right_top(), color);
            mesh.colored_vertex(rect.left_bottom(), darken);
            mesh.colored_vertex(rect.right_bottom(), darken);
        }
    }
}
