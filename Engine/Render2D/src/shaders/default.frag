#version 420

layout(location = 0) out vec4 outColor;

layout(location = 0) in vec2 inTexCoords;
layout(location = 1) in vec4 inColor;

layout(set = 2, binding = 0) uniform sampler2D atlas;

void main() {
    vec4 tex = texture(atlas, inTexCoords);
    outColor = vec4(mix(tex.rgb, inColor.rgb, inColor.a), tex.a);
}