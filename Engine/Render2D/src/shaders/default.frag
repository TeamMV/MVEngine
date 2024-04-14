#version 420

layout(location = 0) out vec4 outColor;

layout(location = 0) in vec2 inTexCoords;

layout(set = 2, binding = 0) uniform sampler2D atlas;

void main() {
    outColor = texture(atlas, inTexCoords);
}