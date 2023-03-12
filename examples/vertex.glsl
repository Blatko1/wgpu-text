#version 450

layout(location = 0) in vec3 pos;

layout(set = 0, binding = 0) uniform GlobalMatrix {
    mat4 proj_view;
};

void main() {
    gl_Position = proj_view * vec4(pos, 1.0);
}