#version 420

struct vertexOutStruct {
    vec2 size;
    vec4 color;
    int texID;
    vec4 texRegion;
};

struct geometryOutStruct {
    vec4 color;
    int texID;
    vec2 texCoords;
};

layout(location = 0) in vertexOutStruct vertexOut[];
layout(location = 0) out geometryOutStruct geometryOut;

layout(set = 0, binding = 0) uniform Matrices {
    mat4 view;
    mat4 proj;
} mat;

layout (points) in;
layout (triangle_strip, max_vertices = 4) out;

void main() {
    vec4 position = gl_in[gl_InvocationID].gl_Position;
    vertexOutStruct vertex = vertexOut[gl_InvocationID];

    vec4 vertices[4];
    vertices[0] = position + vec4(0.0);
    vertices[1] = position + vec4(vertex.size.x, 0.0, 0.0, 0.0);
    vertices[2] = position + vec4(0.0, vertex.size.y, 0.0, 0.0);
    vertices[3] = position + vec4(vertex.size.x, vertex.size.y, 0.0, 0.0);
    vec2 texCoord[4];
    texCoord[0] = vec2(vertex.texRegion[0], vertex.texRegion[1]);
    texCoord[1] = vec2(vertex.texRegion[2], vertex.texRegion[1]);
    texCoord[2] = vec2(vertex.texRegion[0], vertex.texRegion[3]);
    texCoord[3] = vec2(vertex.texRegion[2], vertex.texRegion[3]);

    geometryOut.texID = vertex.texID;
    geometryOut.color = vertex.color;

    for (int i = 0; i < 4; i++) {
        gl_Position = mat.proj * vertices[i];
        geometryOut.texCoords = texCoord[i];
        EmitVertex();
    }

    EndPrimitive();
}