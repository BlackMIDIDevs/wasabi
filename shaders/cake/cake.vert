#version 450

// struct CakeVertex2 {
//     left: f32,
//     right: f32,
//     start: i32,
//     end: i32,
//     buffer_index: i32,
// }

layout(location = 0) in float left;
layout(location = 1) in float right;
layout(location = 2) in int start;
layout(location = 3) in int end;
layout(location = 4) in int buffer_index;

layout(location = 0) out float v_left;
layout(location = 1) out float v_right;
layout(location = 2) out int v_start;
layout(location = 3) out int v_end;
layout(location = 4) out int v_buffer_index;


// layout(location = 0) in vec2 uv;
// layout(location = 1) in vec2 left_right;
// layout(location = 2) in int start;
// layout(location = 3) in int end;
// layout(location = 4) in float x;

// layout(location = 0) out vec2 v_uv;
// layout(location = 1) out vec2 screen_pos;
// layout(location = 2) out vec2 v_left_right;
// layout(location = 3) out int ticks_height;
// layout(location = 4) out int ticks_start;

// layout(push_constant) uniform PushConstants {
//     int start_time;
//     int end_time;
//     int screen_width;
//     int screen_height;
// } consts;

// int tick_at_screen_y(float y) {
//     return int(y * float(consts.end_time - consts.start_time)) + consts.start_time;
// }

// float screen_y_from_tick(int tick) {
//     return float(tick - consts.start_time) / float(consts.end_time - consts.start_time);
// }

// void main() {
//     int bottom_tick = max(consts.start_time, start);
//     int top_tick = min(consts.end_time, end);

//     ticks_height = top_tick - bottom_tick;

//     float start_y = screen_y_from_tick(bottom_tick);
//     float end_y = screen_y_from_tick(top_tick);

//     ticks_start = bottom_tick;

//     float y = 1 - (start_y + (end_y - start_y) * uv.y);

//     vec2 pos = vec2(x, y);
//     screen_pos = pos;

//     pos = pos * 2 - 1;
//     gl_Position = vec4(pos, 0, 1);
//     v_uv = uv;
//     v_left_right = left_right;
// }

void main() {
    v_left = left;
    v_right = right;
    v_start = start;
    v_end = end;
    v_buffer_index = buffer_index;
}