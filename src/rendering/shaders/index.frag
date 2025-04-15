#version 450

#extension GL_EXT_nonuniform_qualifier : enable

precision highp float;

layout (location = 0) in vec4 fColor;
layout (location = 1) in vec2 fUv;
layout (location = 2) in vec2 fRes;
layout (location = 3) in vec3 fFragPos;
layout (location = 4) in float fHasTex;
layout (location = 5) flat in float fTex;

layout(location = 0) out vec4 outColor;

//Shitty a glsl doesnt support indexing thru dynamic, non-uniform values.

uniform sampler2D TEX_SAMPLER_0;
uniform sampler2D TEX_SAMPLER_1;
uniform sampler2D TEX_SAMPLER_2;
uniform sampler2D TEX_SAMPLER_3;
uniform sampler2D TEX_SAMPLER_4;
uniform sampler2D TEX_SAMPLER_5;
uniform sampler2D TEX_SAMPLER_6;
uniform sampler2D TEX_SAMPLER_7;
uniform sampler2D TEX_SAMPLER_8;
uniform sampler2D TEX_SAMPLER_9;
uniform sampler2D TEX_SAMPLER_10;
uniform sampler2D TEX_SAMPLER_11;
uniform sampler2D TEX_SAMPLER_12;
uniform sampler2D TEX_SAMPLER_13;
uniform sampler2D TEX_SAMPLER_14;
uniform sampler2D TEX_SAMPLER_15;

float screenPxRange(vec2 texSize) {
    const float pxRange = 10.0f;

    vec2 unitRange = vec2(pxRange) / vec2(texSize);
    vec2 screenTexSize = vec2(1.0) / fwidth(fUv);
    return max(0.5 * dot(unitRange, screenTexSize), 1.0);
}

float median(float r, float g, float b) {
    return max(min(r, g), min(max(r, g), b));
}

void main() {
    vec4 baseColor;

    if (fHasTex > 0.0) {
        int index = int(fTex);
        vec4 texColor;
        vec2 texSize = vec2(1.0);

        switch (nonuniformEXT(index)) {
            case 0: texColor = texture(TEX_SAMPLER_0, fUv); texSize = textureSize(TEX_SAMPLER_0, 0); break;
            case 1: texColor = texture(TEX_SAMPLER_1, fUv); texSize = textureSize(TEX_SAMPLER_1, 0); break;
            case 2: texColor = texture(TEX_SAMPLER_2, fUv); texSize = textureSize(TEX_SAMPLER_2, 0); break;
            case 3: texColor = texture(TEX_SAMPLER_3, fUv); texSize = textureSize(TEX_SAMPLER_3, 0); break;
            case 4: texColor = texture(TEX_SAMPLER_4, fUv); texSize = textureSize(TEX_SAMPLER_4, 0); break;
            case 5: texColor = texture(TEX_SAMPLER_5, fUv); texSize = textureSize(TEX_SAMPLER_5, 0); break;
            case 6: texColor = texture(TEX_SAMPLER_6, fUv); texSize = textureSize(TEX_SAMPLER_6, 0); break;
            case 7: texColor = texture(TEX_SAMPLER_7, fUv); texSize = textureSize(TEX_SAMPLER_7, 0); break;
            case 8: texColor = texture(TEX_SAMPLER_8, fUv); texSize = textureSize(TEX_SAMPLER_8, 0); break;
            case 9: texColor = texture(TEX_SAMPLER_9, fUv); texSize = textureSize(TEX_SAMPLER_9, 0); break;
            case 10: texColor = texture(TEX_SAMPLER_10, fUv); texSize = textureSize(TEX_SAMPLER_10, 0); break;
            case 11: texColor = texture(TEX_SAMPLER_11, fUv); texSize = textureSize(TEX_SAMPLER_11, 0); break;
            case 12: texColor = texture(TEX_SAMPLER_12, fUv); texSize = textureSize(TEX_SAMPLER_12, 0); break;
            case 13: texColor = texture(TEX_SAMPLER_13, fUv); texSize = textureSize(TEX_SAMPLER_13, 0); break;
            case 14: texColor = texture(TEX_SAMPLER_14, fUv); texSize = textureSize(TEX_SAMPLER_14, 0); break;
            case 15: texColor = texture(TEX_SAMPLER_15, fUv); texSize = textureSize(TEX_SAMPLER_15, 0); break;
            default: texColor = vec4(1.0); break;
        }

        if (fHasTex == 2.0) {
            vec3 msd = texColor.rgb;
            float sd = median(msd.r, msd.g, msd.b);
            float screenPxDistance = screenPxRange(texSize) * (sd - 0.5);
            float opacity = clamp(screenPxDistance + 0.5, 0.0, 1.0);

            if (opacity <= 0) discard;

            baseColor = vec4(vec3(fColor.rgb), opacity);
            outColor = baseColor;
        } else {
            baseColor = mix(texColor, vec4(fColor.rgb, texColor.a), fColor.a);
        }
    } else {
        baseColor = fColor;
    }

    if (baseColor.a == 0.0) discard;
    outColor = baseColor;
}
