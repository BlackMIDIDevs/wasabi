#version 450 core

layout(points) in;
layout(triangle_strip, max_vertices = 4) out;

layout(location = 0) in vec2 start_length[];
layout(location = 1) in uint key_color[];

layout(location = 0) out vec3 frag_color;
layout(location = 1) out vec2 frag_tex_coord;
layout(location = 2) out vec2 v_note_size;
layout(location = 3) out vec2 win_size;

layout(push_constant) uniform PushConstants {
    float height_time;
    float win_width;
    float win_height;
} consts;

struct KeyPosition {
    float left;
    float right;
};

layout(set = 0, binding = 0) uniform Keys {
    KeyPosition key_positions[256];
};

void main()
{
    float start = start_length[0].x / consts.height_time;
    float end = start + start_length[0].y / consts.height_time;
    start = -(start * 2 - 1);
    end = -(end * 2 - 1);

    uint key = key_color[0] & 0xFF;
    uint col_int = key_color[0] >> 8;

    float col_r = float((col_int >> 16) & 0xFF) / 255.0;
    float col_g = float((col_int >> 8) & 0xFF) / 255.0;
    float col_b = float((col_int >> 0) & 0xFF) / 255.0;
    vec3 color = vec3(col_r, col_g, col_b);

    KeyPosition key_position = key_positions[key];

    float left = key_position.left * 2 - 1;
    float right = key_position.right * 2 - 1;

    vec2 note_size_out = vec2(right - left, start - end);
    vec2 win_size_out = vec2(consts.win_width, consts.win_height);

    gl_Position = vec4(left, start, 0, 1);
    frag_color = color;
    frag_tex_coord = vec2(0, 0);
    v_note_size = note_size_out;
    win_size = win_size_out;
    EmitVertex();

    gl_Position = vec4(right, start, 0, 1);
    frag_color = color;
    frag_tex_coord = vec2(1, 0);
    v_note_size = note_size_out;
    win_size = win_size_out;
    EmitVertex();

    gl_Position = vec4(left, end, 0, 1);
    frag_color = color;
    frag_tex_coord = vec2(0, 1);
    v_note_size = note_size_out;
    win_size = win_size_out;
    EmitVertex();

    gl_Position = vec4(right, end, 0, 1);
    frag_color = color;
    frag_tex_coord = vec2(1, 1);
    v_note_size = note_size_out;
    win_size = win_size_out;
    EmitVertex();

    EndPrimitive();
}
