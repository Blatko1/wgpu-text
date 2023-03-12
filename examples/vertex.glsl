#version 450

layout(location = 0) in vec3 pos;
layout(location = 1) in vec2 tex_pos;

layout(set = 0, binding = 0) uniform GlobalMatrix {
    mat4 proj_view;
};

layout(location = 0) out vec2 texp;

void main() {
    texp = tex_pos;
    gl_Position = proj_view * vec4(pos, 1.0);
}