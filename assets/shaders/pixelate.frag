#version 450

precision highp float;

in vec2 fTexCoord;

out vec4 outColor;

uniform sampler2D tex;
uniform vec2 res;
uniform float time;

uniform float size = 10.0;

void main() {
    vec2 d = 1.0 / res.xy;
    vec2 uv = (d.xy * size) * floor(fTexCoord.xy * res.xy / size);

    outColor = vec4(0);

    for (float i = 0; i < size; i++) {
        for (float j = 0; j < size; j++) {
            outColor += texture(tex, uv.xy + vec2(d.x * i, d.y * j));
        }
    }

    outColor /= pow(size, 2.0);
}