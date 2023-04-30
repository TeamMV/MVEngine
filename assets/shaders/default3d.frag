#version 450

const int MAX_MATERIALS = 100;

struct Material {
    vec4 ambient;
    vec4 diffuse;
    vec4 specular;
    vec4 emmission;

    float alpha;
    float specularExponent;
    float metallic;
    float roughtness;

    int diffuseTextureId;
    int metallicRoughnessTextureId;
    int normalTextureId;
};

in vec3 position;
in int matId;

out vec4 outColor;

uniform Material materials[MAX_MATERIALS];

uniform sampler2D TEX_SAMPLER[GL_MAX_TEXTURE_UNITS];

void main() {
    Material mat = materials[matId];
}