#version 420

#extension GL_EXT_nonuniform_qualifier : enable

layout(location = 0) out vec4 outColor;

layout(location = 0) in vec2 inTexCoords;
layout(location = 1) in vec4 inColor;
layout(location = 2) in flat int texId;
layout(location = 3) in float blending;

layout(set = 2, binding = 0) uniform sampler2D textures[];
layout(set = 2, binding = 1) uniform sampler2D fonts[];

float screenPxRange(sampler2D texture)
{
    const float pxRange = 2.0f;

    vec2 unitRange = vec2(pxRange)/vec2(textureSize(texture, 0));
    vec2 screenTexSize = vec2(1.0)/fwidth(inTexCoords);
    return max(0.5*dot(unitRange, screenTexSize), 1.0);
}

float median(float r, float g, float b)
{
    return max(min(r, g), min(max(r, g), b));
}

void main() {

    if (blending < 0.0f)
    {
        // font
        vec3 msd = texture(fonts[texId], inTexCoords).rgb;
        float sd = median(msd.r, msd.g, msd.b);
        float screenPxDistance = screenPxRange(fonts[texId])*(sd - 0.5);
        float opacity = clamp(screenPxDistance + 0.5, 0.0, 1.0);

        outColor = vec4(texture(fonts[texId], inTexCoords));
    }
    else
    {
        // no font
        outColor = texId == -1 ? inColor : mix(texture(textures[texId], inTexCoords), inColor, blending);
    }
}