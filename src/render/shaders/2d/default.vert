#version 420

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 texCoords;

void main() {
    gl_Position = vec4(pos, 0.0f, 0.0f);
}