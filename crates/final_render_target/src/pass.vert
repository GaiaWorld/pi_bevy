#version 450

#define SHADER_NAME fragment:Final

layout(location = 0) in vec2 a_position;

layout(location = 0) out vec2 vUV;

void main() {
    vUV = a_position + 0.5;
    gl_Position = vec4(a_position * 2., 0., 1.);
}