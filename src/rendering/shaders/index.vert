#version 450

precision highp float;

layout (location = 0) in vec2 translation;
layout (location = 1) in vec2 origin;
layout (location = 2) in vec2 scale;
layout (location = 3) in float rotation;

layout (location = 4) in vec3 pos;
layout (location = 5) in vec4 color;
layout (location = 6) in vec2 uv;
layout (location = 7) in float texture_id;
layout (location = 8) in float has_texture;

uniform mat4 uProjection;
uniform mat4 uView;

uniform float uResX;
uniform float uResY;

layout (location = 0) out vec4 fColor;
layout (location = 1) out vec2 fUv;
layout (location = 2) out vec2 fRes;
layout (location = 3) out vec3 fFragPos;
layout (location = 4) out float fHasTex;
layout (location = 5) flat out float fTex;

void main() {
    fColor = color;
    fUv = uv;
    fTex = texture_id; //uniform texture indexing
    fRes = vec2(uResX, uResY);
    fHasTex = has_texture;

    vec2 vpos = pos.xy;

    mat2 rot;
    rot[0] = vec2(cos(rotation), -sin(rotation));
    rot[1] = vec2(sin(rotation),  cos(rotation));

    vpos -= origin;
    vpos = rot * (vpos * scale);
    vpos += origin;
    vpos += translation;

    fFragPos = vec3(vpos, pos.z);

    gl_Position = uProjection * uView * vec4(vpos, pos.z, 1.0);
}
