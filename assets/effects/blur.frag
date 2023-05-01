#version 450

in vec2 fTexCoord;

out vec4 outColor;

uniform sampler2D tex;
uniform vec2 res;
uniform float time;

uniform float dir = 16.0;
uniform float quality = 4.0;
uniform float size = 8.0;

#define TAU 6.28318530718

void main() {
    vec2 r = size / res.xy;

    vec4 color = texture(tex, fTexCoord);

    for (float d=0.0; d < TAU; d += TAU / dir) {
        for (float i=1.0 / quality; i<=1.0; i+=1.0 / quality) {
            color += texture(tex, fTexCoord + vec2(cos(d), sin(d)) * r * i);
        }
    }

    color /= quality * dir - 15.0;
    outColor = color;
}