#version 450

layout(location = 0) in vec3 position;
layout(location = 1) in vec3 normalVec;
layout(location = 2) in vec2 uv;
layout(location = 3) in float materialId;

layout(location = 0) out vec3 pos;
layout(location = 1) out vec3 normal;
layout(location = 2) out vec2 texCoord;
layout(location = 3) out float matId;

layout(set = 0, binding = 0) uniform UNIFORMS {
    mat4 uProjection;
    mat4 uView;
} uniforms;

layout(std140, set = 1, binding = 0) buffer ModelMatrices {
    mat4 matrices[];
} models;

void main() {
    matId = materialId;
    texCoord = uv;
    normal = normalVec;
    vec4 fragPos = uniforms.uProjection * uniforms.uView * models.matrices[gl_InstanceIndex][int(matId)] * vec4(position, 1.0);
    pos = fragPos.xyz;
    gl_Position = fragPos;
}