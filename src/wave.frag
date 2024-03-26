#version 450

layout(location = 0) in vec2 fTexCoord;

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 0) uniform texture2D tex;
layout(set = 0, binding = 1) uniform sampler sam;
layout(set = 0, binding = 2) uniform UNIFORMS {
    vec2 resolution;
    float time;
} uniforms;


void main() {
    // Time varying pixel color
    const vec2 scale = vec2(5.0);

    vec2 uv = 0.02 * sin(scale * uniforms.time + length(fTexCoord) * 30.0) + fTexCoord;
    outColor = texture(sampler2D(tex, sam), uv).rgba;
}