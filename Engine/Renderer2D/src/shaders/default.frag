#version 450

precision highp float;

layout (location = 0) in vec4 fColor;
layout (location = 1) in vec2 fTexCoords;
layout (location = 2) in float fTexID;
layout (location = 3) in vec2 fRes;
layout (location = 4) in float fIsFont;

out vec4 outColor;

uniform sampler2D TEX_SAMPLER[16];
uniform float uSmoothing;

float sq(float x) {
    return x * x;
}

void main() {
    if (fTexID > 0) {
        if (fIsFont == 1) {
            float distance = texture(TEX_SAMPLER[int(fTexID) - 1], fTexCoords).a;
            float alpha = smoothstep(0.5 - uSmoothing, 0.5 + uSmoothing, distance);
            outColor = vec4(fColor.rgb, fColor.a * alpha);
            return;
        }

        vec4 c = texture(TEX_SAMPLER[int(fTexID) - 1], fTexCoords);

        if (fColor.w > 0.0) {
            outColor = vec4(fColor.x, fColor.y, fColor.z, c.w);
        } else {
            outColor = c;
        }
    }
    else {
        outColor = fColor;
    }
}