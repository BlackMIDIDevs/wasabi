#version 450

layout(location = 0) in vec2 v_uv;
layout(location = 1) in vec2 screen_pos;
layout(location = 2) in vec2 left_right;
layout(location = 3) flat in int ticks_height;
layout(location = 4) flat in int ticks_start;

layout(location = 0) out vec4 fsout_Color;

layout(push_constant) uniform PushConstants {
    int start_time;
    int end_time;
    int screen_width;
    int screen_height;
} consts;

layout (set = 0, binding = 0) readonly buffer BinaryTree
{
    ivec3 BinTree[];
};

// void main()
// {
//     fsout_Color = vec4(1, 1, 1, 1);
// }

// struct KeyLocation {
//     float left;
//     float right;
//     int flags;
//     int _;
// };

// layout(location = 1) in vec2 position;
// layout(location = 0) out vec4 fsout_Color;

// layout (binding = 0) uniform UniformBuffer
// {
//     int width;
//     int height;
//     int _start;
//     int _end;
//     int _keyCount;
// };

// const int start = 0;
// const int end = 1505340;

// const int keyCount = 128;

// layout (binding = 1) readonly buffer BinaryTree
// {
//     ivec3 BinTree[];
// };

// // layout (binding = 2) readonly buffer Colors
// // {
// //     vec4 NoteColor[];
// // };

// // layout (binding = 3) readonly buffer Keys
// // {
// //     KeyLocation KeyLocations[];
// // };

const float border_width = 0.0015;

ivec3 sampleAt(int pos) {
    return BinTree[pos];
}

ivec3 getNoteAt(int time) {
    int nextIndex = sampleAt(0).x;

    int steps = 0;
    while(steps < 100) {
        ivec3 node = sampleAt(nextIndex);

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

    ivec3 note = sampleAt(nextIndex);

    return note;
}

// bool midi_is_white(int p) {
//   float k = fract(p * 5 / 12.0);
//   return 0.1 < k && k < 0.55;
// }

float ticks_to_screen_y(int ticks) {
    float screen_y = float(ticks - consts.start_time) / float(consts.end_time - consts.start_time);
    return screen_y;
}

void main()
{
    // int testKey = int(floor(position.x * keyCount));

    // int whiteKey = -1;
    // int blackKey = -1;

    // for (int i = 0; i < 9; i++) {
    //     int odd = i % 2;
    //     int o = (i - odd) / 2;
    //     if(odd == 1) o = -o;

    //     int k = testKey + o;
    //     if(k < 0 || k >= keyCount) continue;

    //     KeyLocation keyData = KeyLocations[k];
    //     if(keyData.left < position.x && keyData.right >= position.x) {
    //         if(keyData.flags == 1) blackKey = k;
    //         else whiteKey = k;
    //     }
    // }

    // vec4 tex = texture(sampler2D(t_Color, s_Color), position);
    // float mag = length(position-vec2(0.5));
    // fsout_Color = vec4(mix(tex.xyz, vec3(0.0), mag*mag), 1.0);

    // return;

    int time = ticks_start + int(ticks_height * v_uv.y);

    // int key;
    ivec3 note;

    note = getNoteAt(time);

    // fsout_Color = vec4(0, 0, 1, 1) / 10.0 * steps;

    if (note.z == -1) {
        fsout_Color = vec4(0, 0, 0, 0);
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

    // if (!midi_is_white(key)) {
    //     fsout_Color = vec4(1, 1, 1, 1);
    // } else {
    //     fsout_Color = vec4(0, 0, 0, 1);
    // }

    return;

    // if(blackKey == -1 || note.z == -1) {
    //     note = getNoteAt(whiteKey, time);
    //     key = whiteKey;
    // }

    // KeyLocation kdata = KeyLocations[key];

    // float left = 1.0 / keyCount * key;
    // float right = 1.0 / keyCount * (key + 1);

    // if(note.z == -1) {
    //     fsout_Color = vec4(0, 0, 0, 1);
    // }
    // else {
    //     int viewHeight = end - start;

    //     float distFromTop = float(note.y - time);
    //     float distFromBottom = float(time - note.x);

    //     float distFromLeft = float(position.x - left);
    //     float distFromRight = float(right - position.x);

    //     float vdist = min(distFromTop, distFromBottom) / viewHeight / width * height;
    //     float hdist = min(distFromLeft, distFromRight);

    //     float minDist = min(vdist, hdist);

    //     vec4 col = vec4(0, 0, 1, 1);

    //     if(minDist < borderWidth) {
    //         fsout_Color = col * 0.6;
    //     }
    //     else {
    //         fsout_Color = col;
    //     }
    // }
}