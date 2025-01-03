#version 420

#extension GL_EXT_nonuniform_qualifier : enable

layout(location = 0) out vec4 outColor;

layout(location = 0) in vec2 inTexCoords;
layout(location = 1) in vec4 inColor;
layout(location = 2) in float texId;
layout(location = 3) in float blending;

layout(set = 2, binding = 0) uniform sampler2D textures[];
layout(set = 2, binding = 1) uniform sampler2D fonts[];

float screenPxRange(sampler2D texture) {
    const float pxRange = 2.0f;

    vec2 unitRange = vec2(pxRange) / vec2(textureSize(texture, 0));
    vec2 screenTexSize = vec2(1.0) / fwidth(inTexCoords);
    return max(0.5 * dot(unitRange, screenTexSize), 1.0);
}

float median(float r, float g, float b) {
    return max(min(r, g), min(max(r, g), b));
}

void main() {
    int iTexId = int(texId);
    if (blending < 0.0f) {
        // font
        vec3 msd = texture(fonts[iTexId], inTexCoords).rgb;
        float sd = median(msd.r, msd.g, msd.b);
        float screenPxDistance = screenPxRange(fonts[iTexId]) * (sd - 0.5);
        float opacity = clamp(screenPxDistance + 0.5, 0.0, 1.0);

        if (opacity <= 0) discard;

        outColor = vec4(vec3(1.0f), opacity);
    }
    else {
        // no font
        outColor = iTexId == -1 ? inColor : mix(texture(textures[iTexId], inTexCoords), inColor, blending);
    }
}