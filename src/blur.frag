#version 450

layout(location = 0) in vec2 fTexCoord;

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0) uniform texture2D tex;
layout(set = 0, binding = 1) uniform sampler sam;
layout(set = 0, binding = 2) uniform UNIFORMS {
    vec2 resolution;
    float time;
} uniforms;

const float dir = 16.0;
const float quality = 4.0;
const float size = 8.0;

#define TAU 6.28318530718

void main() {
    vec2 res = vec2(800.0, 600.0);
    vec2 r = size / res.xy;

    vec4 color = texture(sampler2D(tex, sam), fTexCoord);

    for (float d=0.0; d < TAU; d += TAU / dir) {
        for (float i=1.0 / quality; i<=1.0; i+=1.0 / quality) {
            color += texture(sampler2D(tex, sam), fTexCoord + vec2(cos(d), sin(d)) * r * i);
        }
    }

    color /= quality * dir - 15.0;
    outColor = color;
}