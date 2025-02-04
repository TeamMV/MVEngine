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

struct Light {
    vec2 pos;
    vec4 color;
    float intensity;
    float range;
    float falloff;
};

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

uniform Light LIGHTS[50];
uniform int NUM_LIGHTS;
uniform vec4 AMBIENT;

void main() {
    vec4 baseColor;

    if (fHasTex > 0.0) {
        int index = int(fTex);
        vec4 texColor;

        switch (nonuniformEXT(index)) {
            case 0: texColor = texture(TEX_SAMPLER_0, fUv); break;
            case 1: texColor = texture(TEX_SAMPLER_1, fUv); break;
            case 2: texColor = texture(TEX_SAMPLER_2, fUv); break;
            case 3: texColor = texture(TEX_SAMPLER_3, fUv); break;
            case 4: texColor = texture(TEX_SAMPLER_4, fUv); break;
            case 5: texColor = texture(TEX_SAMPLER_5, fUv); break;
            case 6: texColor = texture(TEX_SAMPLER_6, fUv); break;
            case 7: texColor = texture(TEX_SAMPLER_7, fUv); break;
            case 8: texColor = texture(TEX_SAMPLER_8, fUv); break;
            case 9: texColor = texture(TEX_SAMPLER_9, fUv); break;
            case 10: texColor = texture(TEX_SAMPLER_10, fUv); break;
            case 11: texColor = texture(TEX_SAMPLER_11, fUv); break;
            case 12: texColor = texture(TEX_SAMPLER_12, fUv); break;
            case 13: texColor = texture(TEX_SAMPLER_13, fUv); break;
            case 14: texColor = texture(TEX_SAMPLER_14, fUv); break;
            case 15: texColor = texture(TEX_SAMPLER_15, fUv); break;
            default: texColor = vec4(1.0); break;
        }

        baseColor = mix(texColor, vec4(fColor.rgb, texColor.a), fColor.a);
    } else {
        baseColor = fColor;
    }

    vec3 totalLighting = clamp(AMBIENT.rgb, 0.0, 1.0);

    for(int i = 0; i < NUM_LIGHTS; i++) {
        Light light = LIGHTS[i];
        vec2 positionDelta = light.pos - fFragPos.xy;
        float distance = length(positionDelta);

        if(distance > light.range) continue;

        float normalizedDistance = distance / light.range;
        float attenuation = light.intensity * pow(1.0 - normalizedDistance, light.falloff);

        totalLighting += light.color.rgb * attenuation;
    }

    totalLighting = clamp(totalLighting, 0.0, 1.0);
    outColor = vec4(baseColor.rgb * totalLighting, baseColor.a);
}
