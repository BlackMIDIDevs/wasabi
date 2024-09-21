#version 450

layout(location = 0) in vec3 frag_color;
layout(location = 1) in vec2 frag_tex_coord;
layout(location = 2) in vec2 v_note_size;
layout(location = 3) in vec2 win_size;
layout(location = 4) in flat uint border_width;

layout(location = 0) out vec4 out_color;

const float pi = 3.1415926535897;

void main() {
    vec2 v_uv = frag_tex_coord;
    
    vec3 color = frag_color;
    float aspect = win_size.y / win_size.x;

    color *= (1.0 + cos(pi * 0.5 * v_uv.x)) * 0.5;

    float horiz_width_pixels = v_note_size.x / 2 * win_size.x;
    float vert_width_pixels = v_note_size.y / 2 * win_size.y;

    float horiz_margin = 1 / horiz_width_pixels * border_width;
    float vert_margin = 1 / vert_width_pixels * border_width;

    bool border =
        v_uv.x < horiz_margin ||
        v_uv.x > 1 - horiz_margin ||
        v_uv.y < vert_margin ||
        v_uv.y > 1 - vert_margin;

    if(border)
    {
        color = vec3(frag_color * 0.2);
    }

    // Square for SRGB
    color *= color;
    out_color = vec4(color, 1.0);
}
