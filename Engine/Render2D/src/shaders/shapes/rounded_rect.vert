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

struct DataOut
{
    vec2 scale;
    vec3 rotation;
    float border_radius;
    int smoothness;
    vec4 texCoords; // x, y, width, height,
    vec4 color;
    mat4 view;
    mat4 proj;
};

layout(location = 0) in vec3 pos;

layout(location = 0) out DataOut outData;

layout(set = 0, binding = 0) uniform Matrices {
    mat4 view;
    mat4 proj;
} mat;

struct Transform {
    vec3 position;
    vec3 rotation;
    vec2 scale;
    float border_radius;
    int smoothness;
    vec4 texCoords; // x, y, width, height,
    vec4 color;
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
    mat4 model = translate(t.position.xyz);
    gl_Position = model * vec4(pos.xyz, 1.0f);
    outData.view = mat.view;
    outData.proj = mat.proj;
    outData.smoothness = t.smoothness;
    outData.scale = t.scale;
    outData.rotation = t.rotation;
    outData.border_radius = t.border_radius;
    // outData = TODO
}