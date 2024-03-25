#version 420

layout(set = 0, binding = 0) uniform ColorUniform {
    vec4 color[3];
} ubo;

layout(location = 0) in vec2 pos;

layout(location = 0) out vec4 color;

void main() {
    gl_Position = vec4(pos, 0.0, 1.0);
    color = ubo.color[gl_VertexIndex];
}