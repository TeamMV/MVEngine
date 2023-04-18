#version 450

precision highp float;

in vec2 fTexCoord;

out vec4 outColor;

uniform sampler2D tex;

void main() {
    //outColor = vec4(1.0, 0.0, 0.0, 1.0);
    outColor = texture(tex, fTexCoord);
}