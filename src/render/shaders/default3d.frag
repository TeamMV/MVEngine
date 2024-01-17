#version 450

struct Material {
    vec4 ambient;
    vec4 diffuse;
    vec4 specular;
    vec4 emmission;

    float alpha;
    float specularExponent;
    float metallic;
    float roughness;

    float diffuseTextureId;
    float metallicRoughnessTextureId;
    float normalTextureId;
};

layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 texCoord;
layout(location = 3) in float matId;

layout(location = 0) out vec4 outColor;

layout(set = 2, binding = 0) uniform UNIFORMS {
    Material materials[MAX_MATERIALS];
} uniforms;

layout(set = 0, binding = 1) uniform sampler SAMPLER;
layout(set = 2, binding = 1) uniform texture2D TEXTURES[MAX_TEXTURES];

void main() {
    //do fancy 3d stuff here
    outColor = vec4(1.0);
}