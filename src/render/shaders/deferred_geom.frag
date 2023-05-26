#version 450

layout (location = 0) out vec4 gAlbedoSpec;
layout (location = 1) out vec3 gNormal;
layout (location = 2) out vec3 gPosition;

const int MAX_MATERIALS = 10;

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

layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 texCoord;
layout(location = 3) in flat int matId;

layout(set = 0, binding = 1) uniform UNIFORMS {
    Material materials[MAX_MATERIALS];
    vec4 uCanvasCoords;
    vec2 uCanvasData;
} uniforms;

layout(set = 0, binding = 1) uniform sampler SAMPLER;
layout(set = 2, binding = 0) uniform texture2D TEXTURES[MAX_TEXTURES];

float sq(float x) {
    return x * x;
}

void main() {
    float type = uniforms.uCanvasData.x;
    float r = uniforms.uCanvasData.y;
    vec4 uCanvasCoords = uniforms.uCanvasCoords;
    if (uCanvasCoords.x > gl_FragCoord.x || uCanvasCoords.x + uCanvasCoords.z < gl_FragCoord.x || uCanvasCoords.y > gl_FragCoord.y || uCanvasCoords.y + uCanvasCoords.w < gl_FragCoord.y) {
        discard;
    }
    else if (type != 0 && r > 0) {
        if (type == 1) {
            if (gl_FragCoord.x - uCanvasCoords.x < gl_FragCoord.y - (uCanvasCoords.y + uCanvasCoords.w - r)) {
                discard;
            }
            if ((uCanvasCoords.x + uCanvasCoords.z) - gl_FragCoord.x < gl_FragCoord.y - (uCanvasCoords.y + uCanvasCoords.w - r)) {
                discard;
            }
            if (gl_FragCoord.x - uCanvasCoords.x < (uCanvasCoords.y + r) - gl_FragCoord.y) {
                discard;
            }
            if (gl_FragCoord.x - (uCanvasCoords.x + uCanvasCoords.z - r) > gl_FragCoord.y - uCanvasCoords.y) {
                discard;
            }
        }
        else if (type == 2) {
            if (gl_FragCoord.x < uCanvasCoords.x + r && gl_FragCoord.y > uCanvasCoords.y + uCanvasCoords.w - r && sq((uCanvasCoords.x + r) - gl_FragCoord.x) + sq(gl_FragCoord.y - (uCanvasCoords.y + uCanvasCoords.w - r)) > sq(r)) {
                discard;
            }
            if (gl_FragCoord.x > uCanvasCoords.x + uCanvasCoords.z - r && gl_FragCoord.y > uCanvasCoords.y + uCanvasCoords.w - r && sq(gl_FragCoord.x - (uCanvasCoords.x + uCanvasCoords.z - r)) + sq(gl_FragCoord.y - (uCanvasCoords.y + uCanvasCoords.w - r)) > sq(r)) {
                discard;
            }
            if (gl_FragCoord.x < uCanvasCoords.x + r && gl_FragCoord.y < uCanvasCoords.y + r && sq((uCanvasCoords.x + r) - gl_FragCoord.x) + sq((uCanvasCoords.y + r) - gl_FragCoord.y) > sq(r)) {
                discard;
            }
            if (gl_FragCoord.x > uCanvasCoords.x + uCanvasCoords.z - r && gl_FragCoord.y < uCanvasCoords.y + r && sq(gl_FragCoord.x - (uCanvasCoords.x + uCanvasCoords.z - r)) + sq((uCanvasCoords.y + r) - gl_FragCoord.y) > sq(r)) {
                discard;
            }
        }
    }

    Material mat = uniforms.materials[matId];

    //fancy calculations with material
    gPosition = pos;
    gNormal = normalize(normal);
    if (mat.normalTextureId > 0) {
        gNormal.rgb -= texture(sampler2D(TEXTURES[mat.normalTextureId], SAMPLER), texCoord).rgb;
    }
    gAlbedoSpec.a = mat.metallic;
    if (mat.metallicRoughnessTextureId > 0) {
        gAlbedoSpec.a = texture(sampler2D(TEXTURES[mat.metallicRoughnessTextureId], SAMPLER), texCoord).a;
    }
    gAlbedoSpec.rgb = mat.diffuse.rgb;
    if (mat.diffuseTextureId > 0) {
        gAlbedoSpec.rgb = texture(sampler2D(TEXTURES[mat.diffuseTextureId], SAMPLER), texCoord).rgb;
    }
}