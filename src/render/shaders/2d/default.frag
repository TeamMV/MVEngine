#version 420

struct geometryInStruct {
    vec4 color;
    int texID;
    vec2 texCoords;
};

layout(location = 0) flat in geometryInStruct geometryIn;
layout (location = 0) out vec4 outColor;

void main() {
    outColor = vec4(1.0f);
}