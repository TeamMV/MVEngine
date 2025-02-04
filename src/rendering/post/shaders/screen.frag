#version 450

layout (location = 0) in vec2 fUv;

layout(location = 0) out vec4 outColor;

uniform sampler2D COLOR;
//uniform sampler2D DEPTH;

void main() {
    vec4 color = texture(COLOR, fUv);

    outColor = color;
}