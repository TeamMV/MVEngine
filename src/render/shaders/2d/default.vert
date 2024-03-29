#version 420

layout(set = 0, binding = 0) uniform Matrices {
    mat4 view;
    mat4 proj;
} mat;

struct vertexOutStruct {
    vec4 color;
    int texID;
    vec2 texCoords;
};


layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 texCoords;
layout(location = 2) in vec4 color;
layout(location = 3) in float texID;

layout(location = 0) out vertexOutStruct vertexOut;

void main() {
    vertexOut.color = color;

    gl_Position = vec4(pos, 0.0f, 0.0f);
}