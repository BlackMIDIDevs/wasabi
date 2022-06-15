#version 450 core

layout(points) in;
layout(triangle_strip, max_vertices = 4) out;

layout(location = 0) in vec2 start_length[];
layout(location = 1) in uint key_color[];

layout(location = 0) out vec4 v_color;

layout(push_constant) uniform PushConstants {
    float height_time;
    float win_width;
    float win_height;
} push_constants;

struct KeyPosition {
    float left;
    float right;
};

layout(set = 0, binding = 0) uniform Keys {
    KeyPosition key_positions[256];
};

void main()
{
    float start = start_length[0].x;
    float end = start_length[0].y;

    uint key = key_color[0];

    KeyPosition key_position = key_positions[key];

    float left = key_position.left * 2 - 1;
    float right = key_position.right * 2 - 1;

    vec4 color = vec4(0.0, 0.0, 1.0, 1.0);

    gl_Position = vec4(left, start, 0, 1);
    v_color = color;
    EmitVertex();

    gl_Position = vec4(right, start, 0, 1);
    v_color = color;
    EmitVertex();

    gl_Position = vec4(left, end, 0, 1);
    v_color = color;
    EmitVertex();

    gl_Position = vec4(right, end, 0, 1);
    v_color = color;
    EmitVertex();

    EndPrimitive();
}