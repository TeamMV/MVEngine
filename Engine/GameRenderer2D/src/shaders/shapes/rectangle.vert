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
layout(location = 2) out int outTexId;
layout(location = 3) out float outBlending;

layout(set = 0, binding = 0) uniform Matrices {
    mat4 view;
    mat4 proj;
    vec2 screenSize;
} mat;

struct Transform {
    vec4 position;
    vec4 rotation;
    vec4 texCoords; // x, y, width, height,
    vec4 color;
    vec2 scale;
    int texId;
    float blending;
};

vec2 GetTextureCoords(int vertexIndex, vec4 inTexCoords)
{
    vec2 texCoords;
    switch (vertexIndex)
    {
        case 0: texCoords = vec2(inTexCoords.x, inTexCoords.y + inTexCoords.w);
            break;
        case 1: texCoords = vec2(inTexCoords.x, inTexCoords.y);
            break;
        case 2: texCoords = vec2(inTexCoords.x + inTexCoords.z, inTexCoords.y);
            break;
        case 3: texCoords = vec2(inTexCoords.x + inTexCoords.z, inTexCoords.y + inTexCoords.w);
            break;
    }

    return texCoords;
}

layout(set = 1, binding = 0, scalar) readonly buffer ObjectUbo
{
    Transform transform[];
} transforms;

void main() {
    Transform t = transforms.transform[gl_InstanceIndex];
    mat4 model = createModelMatrix(t.position.xyz, t.rotation.xyz, t.scale.xy);
    gl_Position = mat.proj * mat.view * model * vec4(pos.xyz, 1.0f);
    outTexCoord = GetTextureCoords(gl_VertexIndex, t.texCoords);
    outColor = t.color;
    outTexId = t.texId;
    outBlending = t.blending;
}