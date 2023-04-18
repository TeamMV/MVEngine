#version 450

precision highp float;

in vec2 fTexCoord;

out vec4 outColor;

uniform sampler2D tex;

void main() {
    outColor = texture(tex, fTexCoord);
}