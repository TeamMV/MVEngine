#version 450

precision highp float;

layout (location = 0) in vec4 fColor;
layout (location = 1) in vec2 fTexCoords;
layout (location = 2) in float fTexID;
layout (location = 3) in float fIsFont;

layout(location = 0) out vec4 outColor;

layout(set = 0, binding = 1) uniform UNIFORMS {
    float smoothing;
} uniforms;
layout(set = 0, binding = 2) uniform sampler SAMPLER;
layout(set = 1, binding = 0) uniform texture2D TEXTURE[MAX_TEXTURES];

void main() {
    int texID = int(fTexID) == 0 ? 0 : int(fTexID) - 1;
    vec4 c = texture(sampler2D(TEXTURE[texID], SAMPLER), fTexCoords);

    if (fTexID > 0) {
        if (fIsFont == 1) {
            float distance = c.a;
            float alpha = smoothstep(0.5 - uniforms.smoothing, 0.5 + uniforms.smoothing, distance);
            outColor = vec4(fColor.rgb, fColor.a * alpha);
            return;
        }

        outColor = mix(c, vec4(mix(c.xyz, fColor.xyz, fColor.w), c.w), ceil(fColor.w));
    }
    else {
        outColor = fColor;
    }
}