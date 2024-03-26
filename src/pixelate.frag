#version 450

layout(location = 0) in vec2 fTexCoord;

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0) uniform texture2D tex;
layout(set = 0, binding = 1) uniform sampler sam;
layout(set = 0, binding = 2) uniform UNIFORMS {
    vec2 resolution;
    float time;
} uniforms;

layout(set = 1, binding = 0) uniform CUSTOM {
    float size;
} custom;

void main() {
    vec2 d = 1.0 / uniforms.resolution.xy;
    vec2 uv = (d.xy * custom.size) * floor(fTexCoord.xy * uniforms.resolution.xy / custom.size);

    outColor = vec4(0);

    for (float i = 0; i < custom.size; i++) {
        for (float j = 0; j < custom.size; j++) {
            outColor += texture(sampler2D(tex, sam), uv.xy + vec2(d.x * i, d.y * j));
        }
    }

    outColor /= pow(custom.size, 2.0);
}