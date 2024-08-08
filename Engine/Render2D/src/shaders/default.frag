#version 420

layout(location = 0) out vec4 outColor;

layout(location = 0) in vec2 inTexCoords;
layout(location = 1) in vec4 inColor;
layout(location = 2) in flat int texId;
layout(location = 3) in float blending;

layout(set = 2, binding = 0) uniform sampler2D textures[];
layout(set = 2, binding = 1) uniform sampler2D fonts[];

void main() {

    if (blending < 0.0f)
    {
        // font
    }
    else
    {
        // no font
        vec4 tex = texture(textures[0], inTexCoords);
        outColor = texId == -1 ? inColor : mix(tex, inColor, blending);
    }
}