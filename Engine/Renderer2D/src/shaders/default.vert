#version 460

#extension GL_EXT_scalar_block_layout : enable

precision highp float;

struct Transform {
    vec2 translation;
    vec2 origin;
    vec2 scale;
    float rotation;
    int _align;
};

struct Triangle {
    Transform transform;
    Transform canvasTransform;
    vec4 colors[3];
    vec2 points[3];
    vec2 texCoords[3];
    float z;
    float texId;
    float blending;
    int _align;
};

layout(location = 0) out vec2 outTexCoord;
layout(location = 1) out vec4 outColor;
layout(location = 2) out float outTexId;
layout(location = 3) out float outBlending;

layout(set = 0, binding = 0) uniform Matrices {
    mat4 view;
    mat4 proj;
    vec2 screenSize;
} mat;

layout(set = 1, binding = 0, scalar) readonly buffer ObjectUbo {
    Triangle triangles[];
} triangles;

void main() {
    Triangle tri = triangles.triangles[gl_InstanceIndex];
    vec2 pos = tri.points[gl_VertexIndex];
    Transform transform = tri.transform;
    Transform canvasTransform = tri.canvasTransform;

    mat2 rot;
    rot[0] = vec2(cos(transform.rotation), 0 - sin(transform.rotation));
    rot[1] = vec2(sin(transform.rotation), cos(transform.rotation));
    pos -= transform.origin;
    pos = rot * (pos * transform.scale);
    pos += transform.origin;

    mat2 canvasRot;
    canvasRot[0] = vec2(cos(canvasTransform.rotation), 0 - sin(canvasTransform.rotation));
    canvasRot[1] = vec2(sin(canvasTransform.rotation), cos(canvasTransform.rotation));
    pos -= canvasTransform.origin;
    pos = canvasRot * (pos * canvasTransform.scale);
    pos += canvasTransform.origin;

    pos = pos + canvasTransform.translation + transform.translation;

    gl_Position = mat.proj * mat.view * vec4(pos, tri.z, 1.0);

    outTexCoord = tri.texCoords[gl_VertexIndex];
    outColor = tri.colors[gl_VertexIndex];
    outTexId = tri.texId;
    outBlending = tri.blending;
}