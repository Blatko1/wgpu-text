#version 450

layout(location = 0) in vec2 texp;

layout(location = 0) out vec4 color;

layout(set = 0, binding = 1)
uniform texture2D t;

layout(set = 0, binding = 2)
uniform sampler s;

void main() {
    color = texture(sampler2D(t, s), texp);
}