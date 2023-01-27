use egui::{Color32, Mesh, Pos2, Rect, Sense, Ui};

use crate::midi::MIDIColor;

use super::keyboard_layout::KeyboardView;

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
        let key_density =
            ((rect.width() / key_view.visible_range.len() as f32) / 15.0).clamp(1.0, 5.0);
        let onepx = ui.painter().round_to_pixel(key_density);

        let md_height = rect.height() * 0.048;
        let bar = rect.height() * 0.06;

        let black_key_overlap = bar / 2.35;
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
                    let darkened = Color32::from_rgb(
                        (color.r() as f32 * 0.6) as u8,
                        (color.g() as f32 * 0.6) as u8,
                        (color.b() as f32 * 0.6) as u8,
                    );
                    let darkened2 = Color32::from_rgb(
                        (color.r() as f32 * 0.3) as u8,
                        (color.g() as f32 * 0.3) as u8,
                        (color.b() as f32 * 0.3) as u8,
                    );

                    add_rect_triangles(&mut mesh);
                    mesh.colored_vertex(Pos2::new(map_x(key.left), top), darkened2);
                    mesh.colored_vertex(Pos2::new(map_x(key.right), top), darkened2);
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.left), top + black_key_overlap),
                        darkened,
                    );
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.right), top + black_key_overlap),
                        darkened,
                    );

                    add_rect_triangles(&mut mesh);
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.left), top + black_key_overlap),
                        darkened,
                    );
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.right), top + black_key_overlap),
                        darkened,
                    );
                    mesh.colored_vertex(Pos2::new(map_x(key.left), bottom), color);
                    mesh.colored_vertex(Pos2::new(map_x(key.right), bottom), color);

                    add_rect_triangles(&mut mesh);
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.left), bottom - key_density * 2.0),
                        darkened2,
                    );
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.right), bottom - key_density * 2.0),
                        darkened2,
                    );
                    mesh.colored_vertex(Pos2::new(map_x(key.left), bottom), darkened);
                    mesh.colored_vertex(Pos2::new(map_x(key.right), bottom), darkened);
                } else {
                    // Not pressed
                    add_rect_triangles(&mut mesh);
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.left), top),
                        Color32::from_rgb(110, 110, 110),
                    );
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.right), top),
                        Color32::from_rgb(110, 110, 110),
                    );
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.left), top + black_key_overlap),
                        Color32::from_rgb(210, 210, 210),
                    );
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.right), top + black_key_overlap),
                        Color32::from_rgb(210, 210, 210),
                    );

                    add_rect_triangles(&mut mesh);
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.left), top + black_key_overlap),
                        Color32::from_rgb(210, 210, 210),
                    );
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.right), top + black_key_overlap),
                        Color32::from_rgb(210, 210, 210),
                    );
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.left), bottom - md_height),
                        Color32::WHITE,
                    );
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.right), bottom - md_height),
                        Color32::WHITE,
                    );

                    add_rect_triangles(&mut mesh);
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.left), bottom - md_height),
                        Color32::from_rgb(190, 190, 190),
                    );
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.right), bottom - md_height),
                        Color32::from_rgb(190, 190, 190),
                    );
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.left), bottom),
                        Color32::from_rgb(120, 120, 120),
                    );
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.right), bottom),
                        Color32::from_rgb(120, 120, 120),
                    );

                    add_rect_triangles(&mut mesh);
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.left), bottom - md_height),
                        Color32::from_rgb(70, 70, 70),
                    );
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.right), bottom - md_height),
                        Color32::from_rgb(70, 70, 70),
                    );
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.left), bottom - md_height + key_density * 2.0),
                        Color32::from_rgb(140, 140, 140),
                    );
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.right), bottom - md_height + key_density * 2.0),
                        Color32::from_rgb(140, 140, 140),
                    );
                }
                // White key borders
                let color4 = Color32::from_rgb(40, 40, 40);
                let top_left4 = Pos2::new(map_x(key.right), top);
                let bottom_right4 = Pos2::new(map_x(key.right) - onepx, bottom);
                let rect4 = Rect::from_min_max(top_left4, bottom_right4);
                mesh.add_colored_rect(rect4, color4);
            }
        }

        // Coloured bar
        let bar_color_dark = Color32::from_rgb(
            (bar_color.r() as f32 * 0.3) as u8,
            (bar_color.g() as f32 * 0.3) as u8,
            (bar_color.b() as f32 * 0.3) as u8,
        );
        add_rect_triangles(&mut mesh);
        mesh.colored_vertex(
            Pos2::new(rect.left(), top - black_key_overlap),
            bar_color_dark,
        );
        mesh.colored_vertex(
            Pos2::new(rect.right(), top - black_key_overlap),
            bar_color_dark,
        );
        mesh.colored_vertex(Pos2::new(rect.left(), top), *bar_color);
        mesh.colored_vertex(Pos2::new(rect.right(), top), *bar_color);

        // Progress bar
        add_rect_triangles(&mut mesh);
        mesh.colored_vertex(
            Pos2::new(rect.left(), rect.top()),
            Color32::from_rgb(90, 90, 90),
        );
        mesh.colored_vertex(
            Pos2::new(rect.right(), rect.top()),
            Color32::from_rgb(90, 90, 90),
        );
        mesh.colored_vertex(
            Pos2::new(rect.left(), top - black_key_overlap),
            Color32::from_rgb(40, 40, 40),
        );
        mesh.colored_vertex(
            Pos2::new(rect.right(), top - black_key_overlap),
            Color32::from_rgb(40, 40, 40),
        );

        for (i, key) in key_view.iter_visible_keys() {
            if key.black {
                if let Some(color) = colors[i].map(map_color) {
                    // Pressed
                    let darkened = Color32::from_rgb(
                        (color.r() as f32 * 0.76) as u8,
                        (color.g() as f32 * 0.76) as u8,
                        (color.b() as f32 * 0.76) as u8,
                    );

                    let lightened = Color32::from_rgb(
                        (color.r() as f32 * 1.3) as u8,
                        (color.g() as f32 * 1.3) as u8,
                        (color.b() as f32 * 1.3) as u8,
                    );

                    let md_height = md_height / 2.0;
                    let black_key_overlap = black_key_overlap / 2.2;

                    add_rect_triangles(&mut mesh);
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.left) + key_density, black_bottom - md_height),
                        color,
                    );
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.right) - key_density, black_bottom - md_height),
                        color,
                    );
                    mesh.colored_vertex(Pos2::new(map_x(key.left), black_bottom), darkened);
                    mesh.colored_vertex(Pos2::new(map_x(key.right), black_bottom), darkened);

                    add_rect_triangles(&mut mesh);
                    mesh.colored_vertex(Pos2::new(map_x(key.left), top), lightened);
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.left) + 2.0 * key_density, top - black_key_overlap),
                        darkened,
                    );
                    mesh.colored_vertex(Pos2::new(map_x(key.left), black_bottom), lightened);
                    mesh.colored_vertex(
                        Pos2::new(
                            map_x(key.left) + 2.0 * key_density,
                            black_bottom - md_height,
                        ),
                        darkened,
                    );

                    add_rect_triangles(&mut mesh);
                    mesh.colored_vertex(
                        Pos2::new(
                            map_x(key.right) - 2.0 * key_density,
                            top - black_key_overlap,
                        ),
                        lightened,
                    );
                    mesh.colored_vertex(Pos2::new(map_x(key.right), top), darkened);
                    mesh.colored_vertex(
                        Pos2::new(
                            map_x(key.right) - 2.0 * key_density,
                            black_bottom - md_height,
                        ),
                        lightened,
                    );
                    mesh.colored_vertex(Pos2::new(map_x(key.right), black_bottom), darkened);

                    add_rect_triangles(&mut mesh);
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.left) + 2.0 * key_density, top - black_key_overlap),
                        color,
                    );
                    mesh.colored_vertex(
                        Pos2::new(
                            map_x(key.right) - 2.0 * key_density,
                            top - black_key_overlap,
                        ),
                        color,
                    );
                    mesh.colored_vertex(
                        Pos2::new(
                            map_x(key.left) + 2.0 * key_density,
                            black_bottom - md_height,
                        ),
                        darkened,
                    );
                    mesh.colored_vertex(
                        Pos2::new(
                            map_x(key.right) - 2.0 * key_density,
                            black_bottom - md_height,
                        ),
                        darkened,
                    );
                } else {
                    // Not pressed
                    add_rect_triangles(&mut mesh);
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.left) + key_density, black_bottom - md_height),
                        Color32::from_rgb(105, 105, 105),
                    );
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.right) - key_density, black_bottom - md_height),
                        Color32::from_rgb(105, 105, 105),
                    );
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.left), black_bottom),
                        Color32::from_rgb(20, 20, 20),
                    );
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.right), black_bottom),
                        Color32::from_rgb(20, 20, 20),
                    );

                    add_rect_triangles(&mut mesh);
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.left), top),
                        Color32::from_rgb(20, 20, 20),
                    );
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.left) + 2.0 * key_density, top - black_key_overlap),
                        Color32::from_rgb(105, 105, 105),
                    );
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.left), black_bottom),
                        Color32::from_rgb(20, 20, 20),
                    );
                    mesh.colored_vertex(
                        Pos2::new(
                            map_x(key.left) + 2.0 * key_density,
                            black_bottom - md_height,
                        ),
                        Color32::from_rgb(105, 105, 105),
                    );

                    add_rect_triangles(&mut mesh);
                    mesh.colored_vertex(
                        Pos2::new(
                            map_x(key.right) - 2.0 * key_density,
                            top - black_key_overlap,
                        ),
                        Color32::from_rgb(105, 105, 105),
                    );
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.right), top),
                        Color32::from_rgb(20, 20, 20),
                    );
                    mesh.colored_vertex(
                        Pos2::new(
                            map_x(key.right) - 2.0 * key_density,
                            black_bottom - md_height,
                        ),
                        Color32::from_rgb(105, 105, 105),
                    );
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.right), black_bottom),
                        Color32::from_rgb(20, 20, 20),
                    );

                    add_rect_triangles(&mut mesh);
                    mesh.colored_vertex(
                        Pos2::new(map_x(key.left) + 2.0 * key_density, top - black_key_overlap),
                        Color32::from_rgb(20, 20, 20),
                    );
                    mesh.colored_vertex(
                        Pos2::new(
                            map_x(key.right) - 2.0 * key_density,
                            top - black_key_overlap,
                        ),
                        Color32::from_rgb(20, 20, 20),
                    );
                    mesh.colored_vertex(
                        Pos2::new(
                            map_x(key.left) + 2.0 * key_density,
                            black_bottom - md_height,
                        ),
                        Color32::from_rgb(40, 40, 40),
                    );
                    mesh.colored_vertex(
                        Pos2::new(
                            map_x(key.right) - 2.0 * key_density,
                            black_bottom - md_height,
                        ),
                        Color32::from_rgb(40, 40, 40),
                    );
                }
            }
        }

        ui.painter().add(mesh);
    }
}

fn add_rect_triangles(mesh: &mut Mesh) {
    let idx = mesh.vertices.len() as u32;
    mesh.add_triangle(idx, idx + 1, idx + 2);
    mesh.add_triangle(idx + 2, idx + 1, idx + 3);
}
