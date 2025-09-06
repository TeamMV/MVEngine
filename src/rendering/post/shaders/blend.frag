#version 450

layout (location = 0) in vec2 fUv;

layout(location = 0) out vec4 outColor;

uniform sampler2D DST;
uniform sampler2D SRC;

void main() {
    vec4 color = texture(COLOR, fUv);

    outColor = color;
}