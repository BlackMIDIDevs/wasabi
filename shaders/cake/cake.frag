#version 450

layout(location = 0) in vec2 v_uv;
layout(location = 1) in vec2 screen_pos;
layout(location = 2) in vec2 left_right;
layout(location = 3) flat in int ticks_height;
layout(location = 4) flat in int ticks_start;
layout(location = 5) flat in int buffer_index;

layout(location = 0) out vec4 fsout_Color;

layout(push_constant) uniform PushConstants {
    int start_time;
    int end_time;
    int screen_width;
    int screen_height;
} consts;

layout(set = 0, binding = 0) readonly buffer BufferArray
{
    ivec4 BinTree[];
} buffers[256];

const float border_width = 0.0015;

ivec4 getNoteAt(int time) {
    int nextIndex = buffers[buffer_index].BinTree[0].x;

    int steps = 0;
    while(steps < 100) {
        ivec4 node = buffers[buffer_index].BinTree[nextIndex];

        int offset = 0;
        if(time < node.x) offset = node.y;
        else offset = node.z;

        if (offset > 0) {
            nextIndex -= offset;
            break;
        }
        nextIndex += offset;

        steps++;
    }

    ivec4 note = buffers[buffer_index].BinTree[nextIndex];

    return note;
}

float ticks_to_screen_y(int ticks) {
    float screen_y = float(ticks - consts.start_time) / float(consts.end_time - consts.start_time);
    return screen_y;
}

void main()
{
    int time = ticks_start + int(ticks_height * v_uv.y);

    ivec4 note = getNoteAt(time);

    if (note.z == -1) {
        discard;
    } else {
        fsout_Color = vec4(((note.z >> 16) & 0xFF) / 255.0, ((note.z >> 8) & 0xFF) / 255.0, (note.z & 0xFF) / 255.0, 1);
    }

    float note_top = ticks_to_screen_y(note.x);
    float note_bottom = ticks_to_screen_y(note.y);

    float y_multiplier = float(consts.screen_height) / float(consts.screen_width);

    float y = 1 - screen_pos.y;
    float note_top_dist = (y - note_top) * y_multiplier;
    float note_bottom_dist = (note_bottom - y) * y_multiplier;

    float note_left_dist = (screen_pos.x - left_right.x);
    float note_right_dist = (left_right.y - screen_pos.x);

    float min_x_dist = min(note_left_dist, note_right_dist);
    float min_y_dist = min(note_top_dist, note_bottom_dist);
    float min_dist = min(min_x_dist, min_y_dist);

    if(min_dist < border_width) {
        fsout_Color = fsout_Color * 0.2;
    }
}