#version 450

precision highp float;

layout (location = 0) in vec3 aVertPos;
layout (location = 1) in float aRotation;
layout (location = 2) in vec2 aRotationOrigin;
layout (location = 3) in vec4 aColor;
layout (location = 4) in vec2 aTexCoords;
layout (location = 5) in float aTexID;
layout (location = 6) in vec4 aCanvasCoords;
layout (location = 7) in vec2 aCanvasData;
layout (location = 8) in float aUseCam;
out vec4 fColor;
out vec2 fTexCoords;
out float fTexID;
out vec4 fCanvasCoords;
out vec2 fCanvasData;
out vec2 fRes;

uniform mat4 uProjection;
uniform mat4 uView;

uniform float uResX;
uniform float uResY;

void main() {
    fColor = aColor;
    fTexCoords = aTexCoords;
    fTexID = aTexID;
    fCanvasCoords = aCanvasCoords;
    fCanvasData = aCanvasData;
    fRes = vec2(uResX, uResY);

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
        gl_Position = uProjection * uView * vec4(pos, aVertPos.z, 1.0);
    } else {
        gl_Position = uProjection * vec4(pos, aVertPos.z, 1.0);
    }
}