#version 450

layout(location = 0) in vec2 fTexCoord;

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0) uniform texture2D tex;
layout(set = 0, binding = 1) uniform sampler sam;
//uniform vec2 res;
//uniform float time;

const float size = 10.0;

void main() {
    vec2 res = vec2(800, 600);
    vec2 d = 1.0 / res.xy;
    vec2 uv = (d.xy * size) * floor(fTexCoord.xy * res.xy / size);

    outColor = vec4(0);

    for (float i = 0; i < size; i++) {
        for (float j = 0; j < size; j++) {
            outColor += texture(sampler2D(tex, sam), uv.xy + vec2(d.x * i, d.y * j));
        }
    }

    outColor /= pow(size, 2.0);
}