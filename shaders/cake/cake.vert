#version 450

layout(location = 0) in float left;
layout(location = 1) in float right;
layout(location = 2) in int start;
layout(location = 3) in int end;
layout(location = 4) in int buffer_index;
layout(location = 5) in int border_width;

layout(location = 0) out float v_left;
layout(location = 1) out float v_right;
layout(location = 2) out int v_start;
layout(location = 3) out int v_end;
layout(location = 4) out int v_buffer_index;
layout(location = 5) out int v_border_width;

void main() {
    v_left = left;
    v_right = right;
    v_start = start;
    v_end = end;
    v_buffer_index = buffer_index;
    v_border_width = border_width;
}