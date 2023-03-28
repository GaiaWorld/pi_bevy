#version 450

#define SHADER_NAME fragment:Final

layout(location = 0) in vec2 vUV;

layout(location = 0) out vec4 gl_FragColor;

layout(set = 0, binding = 0) uniform texture2D u_texture;
layout(set = 0, binding = 1) uniform sampler sampler_u_texture;

void main() {
    gl_FragColor = texture(sampler2D(u_texture, sampler_u_texture), vUV);
}