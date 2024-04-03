#version 420

layout(location = 0) in vec3 pos;
layout(location = 1) in vec2 texCoords;

layout(set = 0, binding = 0) uniform Matrices {
    mat4 view;
    mat4 proj;
} mat;

void main() {
    gl_Position = mat.proj * mat.view * vec4(pos, 1.0f);
}