#version 420

struct geometryOutStruct {
    vec4 color;
    int texID;
    vec2 texCoords;
};

layout(location = 0) flat in geometryOutStruct geometryOut;
layout (location = 0) out vec4 outColor;

void main() {
    outColor = geometryOut.color;
}