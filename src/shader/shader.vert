#version 450
#pragma shader_stage(vertex)

layout(location = 0) in vec2 a_pos;

void main() {
    gl_Position = vec4((a_pos / (vec2(800.0, 600) * 1.5)) * 2 - 1, 0.0, 1.0);
    gl_PointSize = 5.0;
}