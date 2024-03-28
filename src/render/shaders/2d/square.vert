#version 420

struct vertexOutStruct {
    vec2 size;
    vec4 color;
    int texID;
    vec4 texRegion;
};

layout(location = 0) in vec2 pos;
layout(location = 1) in vec2 size;
layout(location = 2) in vec4 color;
layout(location = 3) in float texID;
layout(location = 4) in vec4 texRegion;

layout(location = 0) out vertexOutStruct vertexOut;

layout(set = 0, binding = 0) uniform Matrices {
    mat4 view;
    mat4 proj;
} mat;

void main() {
    gl_Position = mat.proj * mat.view * vec4(pos, 0.0, 1.0);
    vertexOut.size = size;
    vertexOut.color = color;
    vertexOut.texID = int(texID);
    vertexOut.texRegion = texRegion;
}