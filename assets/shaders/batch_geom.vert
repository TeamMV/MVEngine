#version 450

layout(location=0) in vec3 position;
layout(location=1) in vec3 normalVec;
layout(location=2) in int materialId;
layout(location=3) in vec2 uv;

out vec3 pos;
out vec3 normal;
out vec2 texCoord;
out int matId;

uniform mat4 uProjection;
uniform mat4 uView;
uniform mat4 uModel;

void main() {
    matId = materialId;
    texCoord = uv;
    normal = normalVec;
    vec4 fragPos = uProjection * uView * uModel * vec4(position, 1.0);
    pos = fragPos.xyz;
    gl_Position = fragPos;
}