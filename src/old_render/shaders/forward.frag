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

    Material mat = uniforms.materials[int(matId)];

    //fancy calculations with material
    outColor = vec4(mix(mat.diffuse.rgb, texture(sampler2D(TEXTURES[int(mat.diffuseTextureId - 1)], SAMPLER), texCoord).rgb, min(mat.diffuseTextureId, 1.0)), 1.0);
    outColor = vec4(1.0);
}