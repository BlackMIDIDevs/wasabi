#version 430 core

layout(points) in;
layout(triangle_strip, max_vertices = 4) out;

layout(location = 0) in vec2 position[];
layout(location = 1) in vec4 color[];

layout(location = 0) out vec4 v_color;

void main()
{
    gl_Position = vec4(position[0] + vec2(0.1, 0.1), 0, 1);
    v_color = color[0];
    EmitVertex();

    gl_Position = vec4(position[0] + vec2(-0.1, 0.1), 0, 1);
    v_color = color[0];
    EmitVertex();

    gl_Position = vec4(position[0] + vec2(0.1, -0.1), 0, 1);
    v_color = color[0];
    EmitVertex();

    gl_Position = vec4(position[0] + vec2(-0.1, -0.1), 0, 1);
    v_color = color[0];
    EmitVertex();

    EndPrimitive();
}