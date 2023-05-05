#version 450

precision highp float;

layout(location = 0) in vec3 aVertPos;
layout(location = 1) in float aRotation;
layout(location = 2) in vec2 aRotationOrigin;
layout(location = 3) in vec4 aColor;
layout(location = 4) in vec2 aTexCoords;
layout(location = 5) in float aTexID;
layout(location = 6) in vec4 aCanvasCoords;
layout(location = 7) in vec2 aCanvasData;
layout(location = 8) in float aUseCam;

layout(location = 0) out vec4 fColor;
layout(location = 1) out vec2 fTexCoords;
layout(location = 2) out float fTexID;
layout(location = 3) out vec4 fCanvasCoords;
layout(location = 4) out vec2 fCanvasData;

layout(set = 0, binding = 0) uniform UNIFORMS {
    mat4 proj;
    mat4 view;
} uniforms;

void main() {
    fColor = aColor;
    fTexCoords = aTexCoords;
    fTexID = aTexID;
    fCanvasCoords = aCanvasCoords;
    fCanvasData = aCanvasData;

    vec2 pos = aVertPos.xy;

    if (aRotation != 0) {
        mat2 rot;
        rot[0] = vec2(cos(aRotation), - sin(aRotation));
        rot[1] = vec2(sin(aRotation), cos(aRotation));
        pos -= aRotationOrigin;
        pos = rot * pos;
        pos += aRotationOrigin;
    }

    if (aUseCam == 1) {
        gl_Position = uniforms.proj * uniforms.view * vec4(pos, aVertPos.z, 1.0);
    } else {
        gl_Position = uniforms.proj * vec4(pos, aVertPos.z, 1.0);
    }
}