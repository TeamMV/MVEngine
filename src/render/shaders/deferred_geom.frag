#version 450

layout (location = 0) out vec4 gAlbedoSpec;
layout (location = 1) out vec3 gNormal;
layout (location = 2) out vec3 gPosition;

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
//size 92

layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 texCoord;
layout(location = 3) in float matId;

layout(set = 2, binding = 0) uniform UNIFORMS {
    Material materials[MAX_MATERIALS];
} uniforms;

layout(set = 0, binding = 1) uniform sampler SAMPLER;
layout(set = 2, binding = 1) uniform texture2D TEXTURES[MAX_TEXTURES];

float sq(float x) {
    return x * x;
}

void main() {

    Material mat = uniforms.materials[int(matId)];

    //fancy calculations with material
    gPosition = pos;
    gNormal = normalize(normal);
    gNormal.rgb -= step(0.5, mat.normalTextureId) * texture(sampler2D(TEXTURES[int(mat.normalTextureId - 1)], SAMPLER), texCoord).rgb;
    gAlbedoSpec.a = mix(mat.metallic, texture(sampler2D(TEXTURES[int(mat.metallicRoughnessTextureId - 1)], SAMPLER), texCoord).a, min(mat.metallicRoughnessTextureId, 1.0));
    gAlbedoSpec.rgb = mix(mat.diffuse.rgb, texture(sampler2D(TEXTURES[int(mat.diffuseTextureId - 1)], SAMPLER), texCoord).rgb, min(mat.diffuseTextureId, 1.0));
}