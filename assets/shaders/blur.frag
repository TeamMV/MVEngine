#version 450

precision highp float;

in vec2 fTexCoord;

out vec4 outColor;

uniform sampler2D tex;
uniform vec2 res;
uniform float time;

float sq(float x) {
    return x * x;
}

void main() {
    outColor = texture(tex, fTexCoord);
    return;
    float Pi = 6.28318530718; // Pi*2

    // GAUSSIAN BLUR SETTINGS {{{
    float dir = 16.0; // BLUR DIRECTIONS (Default 16.0 - More is better but slower)
    float quality = 3.0; // BLUR QUALITY (Default 4.0 - More is better but slower)
    float size = 8.0; // BLUR SIZE (Radius)
    // GAUSSIAN BLUR SETTINGS }}}

    vec2 r = size /res.xy;

    // Pixel colour
    vec4 color = texture(tex, fTexCoord);

    // Blur calculations
    for (float d=0.0; d<Pi; d += Pi / dir) {
        for (float i=1.0/ quality; i<=1.0; i+=1.0/ quality) {
            color += texture(tex, fTexCoord + vec2(cos(d), sin(d)) * r * i);
        }
    }

    // Output to screen
    color /= quality * dir - 15.0;
    outColor = color;
}