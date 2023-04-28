#version 450 core
#extension GL_EXT_nonuniform_qualifier : require

layout(points) in;
layout(triangle_strip, max_vertices = 4) out;

layout(location = 0) in float left[];
layout(location = 1) in float right[];
layout(location = 2) in int start[];
layout(location = 3) in int end[];
layout(location = 4) in int buffer_index[];

layout(location = 0) out vec2 v_uv;
layout(location = 1) out vec2 screen_pos;
layout(location = 2) out vec2 v_left_right;
layout(location = 3) out int ticks_height;
layout(location = 4) out int ticks_start;
layout(location = 5) out int v_buffer_index;

layout(push_constant) uniform PushConstants {
    int start_time;
    int end_time;
    int screen_width;
    int screen_height;
} consts;

int tick_at_screen_y(float y) {
    return int(y * float(consts.end_time - consts.start_time)) + consts.start_time;
}

float screen_y_from_tick(int tick) {
    return float(tick - consts.start_time) / float(consts.end_time - consts.start_time);
}

void main()
{
    // Prepare the shared values

    int bottom_tick = max(consts.start_time, start[0]);
    int top_tick = min(consts.end_time, end[0]);


    float start_y = screen_y_from_tick(bottom_tick);
    float end_y = screen_y_from_tick(top_tick);

    // Prepare the vertices

    vec2 uv;
    float y;
    float x;
    vec2 pos;

    uv = vec2(0, 0);
    y = 1 - (start_y + (end_y - start_y) * uv.y);
    x = left[0];
    pos = vec2(x, y);
    screen_pos = pos;
    pos = pos * 2 - 1;
    gl_Position = vec4(pos, 0, 1);
    v_uv = uv;

    v_buffer_index = buffer_index[0];
    v_left_right = vec2(left[0], right[0]);
    ticks_height = top_tick - bottom_tick;
    ticks_start = bottom_tick;

    EmitVertex();

    uv = vec2(1, 0);
    y = 1 - (start_y + (end_y - start_y) * uv.y);
    x = right[0];
    pos = vec2(x, y);
    screen_pos = pos;
    pos = pos * 2 - 1;
    gl_Position = vec4(pos, 0, 1);
    v_uv = uv;

    v_buffer_index = buffer_index[0];
    v_left_right = vec2(left[0], right[0]);
    ticks_height = top_tick - bottom_tick;
    ticks_start = bottom_tick;

    EmitVertex();

    uv = vec2(0, 1);
    y = 1 - (start_y + (end_y - start_y) * uv.y);
    x = left[0];
    pos = vec2(x, y);
    screen_pos = pos;
    pos = pos * 2 - 1;
    gl_Position = vec4(pos, 0, 1);
    v_uv = uv;

    v_buffer_index = buffer_index[0];
    v_left_right = vec2(left[0], right[0]);
    ticks_height = top_tick - bottom_tick;
    ticks_start = bottom_tick;

    EmitVertex();

    uv = vec2(1, 1);
    y = 1 - (start_y + (end_y - start_y) * uv.y);
    x = right[0];
    pos = vec2(x, y);
    screen_pos = pos;
    pos = pos * 2 - 1;
    gl_Position = vec4(pos, 0, 1);
    v_uv = uv;

    v_buffer_index = buffer_index[0];
    v_left_right = vec2(left[0], right[0]);
    ticks_height = top_tick - bottom_tick;
    ticks_start = bottom_tick;

    EmitVertex();

    EndPrimitive();
}
