#version 450
#pragma shader_stage(fragment)

layout(location=0) out vec4 f_color;

void main() {
    f_color = vec4(0.2, 0.6, 1.0, 1.0);
}