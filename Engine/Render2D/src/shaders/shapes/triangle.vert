#version 460

#extension GL_EXT_scalar_block_layout : enable

mat4 translate(vec3 translation) {
    return mat4(
    vec4(1.0, 0.0, 0.0, 0.0),
    vec4(0.0, 1.0, 0.0, 0.0),
    vec4(0.0, 0.0, 1.0, 0.0),
    vec4(translation, 1.0)
    );
}

mat4 scale(float c)
{
    return mat4(c, 0, 0, 0,
    0, c, 0, 0,
    0, 0, c, 0,
    0, 0, 0, 1);
}

mat4 rotate2d(float a)
{
    return mat4(cos(a), -sin(a), 0, 0,
    sin(a), cos(a), 0, 0,
    0, 0, 1.0, 0,
    0, 0, 0, 1.0);
}

mat4 createModelMatrix(vec3 translation, vec3 rotation, vec2 scale) {
    // Translation matrix
    mat4 translateMatrix = translate(translation);

    // Rotation matrix (using Euler angles)
    mat4 rotateX = mat4(
    1.0, 0.0, 0.0, 0.0,
    0.0, cos(rotation.x), -sin(rotation.x), 0.0,
    0.0, sin(rotation.x), cos(rotation.x), 0.0,
    0.0, 0.0, 0.0, 1.0
    );
    mat4 rotateY = mat4(
    cos(rotation.y), 0.0, sin(rotation.y), 0.0,
    0.0, 1.0, 0.0, 0.0,
    -sin(rotation.y), 0.0, cos(rotation.y), 0.0,
    0.0, 0.0, 0.0, 1.0
    );
    mat4 rotateZ = mat4(
    cos(rotation.z), -sin(rotation.z), 0.0, 0.0,
    sin(rotation.z), cos(rotation.z), 0.0, 0.0,
    0.0, 0.0, 1.0, 0.0,
    0.0, 0.0, 0.0, 1.0
    );
    mat4 rotateMatrix = rotateX * rotateY * rotateZ;

    // Scale matrix
    mat4 scaleMatrix = mat4(
    scale.x, 0.0, 0.0, 0.0,
    0.0, scale.y, 0.0, 0.0,
    0.0, 0.0, 1.0f, 0.0,
    0.0, 0.0, 0.0, 1.0
    );

    // Combine translation, rotation, and scale
    return translateMatrix * rotateMatrix * scaleMatrix;
}

layout(location = 0) in vec3 pos;

layout(location = 0) out vec2 outTexCoord;
layout(location = 1) out vec4 outColor;

layout(set = 0, binding = 0) uniform Matrices {
    mat4 view;
    mat4 proj;
} mat;

struct Transform {
    vec4 vertices[3];
    vec3 translation;
    vec2 scale;
    vec3 rotation;
    vec4 color;
};

layout(set = 1, binding = 0, scalar) readonly buffer ObjectUbo
{
    Transform transform[];
} transforms;

void main() {
    Transform t = transforms.transform[gl_InstanceIndex];
    mat4 model = createModelMatrix(t.translation, t.rotation, t.scale);
    gl_Position = mat.proj * mat.view * model * vec4(vec3(t.vertices[gl_VertexIndex].xy, 1.0), 1.0f);
    outTexCoord = t.vertices[gl_VertexIndex].zw;
    outColor = t.color;
}