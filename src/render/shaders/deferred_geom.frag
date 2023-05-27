#version 450

layout (location = 0) out vec4 gAlbedoSpec;
layout (location = 1) out vec3 gNormal;
layout (location = 2) out vec3 gPosition;

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

    float diffuseTextureId;
    float metallicRoughnessTextureId;
    float normalTextureId;
};

layout(location = 0) in vec3 pos;
layout(location = 1) in vec3 normal;
layout(location = 2) in vec2 texCoord;
layout(location = 3) in float matId;

layout(set = 0, binding = 1) uniform UNIFORMS {
    vec4 uCanvasCoords;
    vec2 uCanvasData;
    Material materials[MAX_MATERIALS];
} uniforms;

layout(set = 0, binding = 2) uniform sampler SAMPLER;
layout(set = 2, binding = 0) uniform texture2D TEXTURES[MAX_TEXTURES];

float sq(float x) {
    return x * x;
}

void main() {
    //float type = uniforms.uCanvasData.x;
    //float r = uniforms.uCanvasData.y;
    //vec4 uCanvasCoords = uniforms.uCanvasCoords;
    //if (uCanvasCoords.x > gl_FragCoord.x || uCanvasCoords.x + uCanvasCoords.z < gl_FragCoord.x || uCanvasCoords.y > gl_FragCoord.y || uCanvasCoords.y + uCanvasCoords.w < gl_FragCoord.y) {
    //    discard;
    //}
    //else if (type != 0 && r > 0) {
    //    if (type == 1) {
    //        if (gl_FragCoord.x - uCanvasCoords.x < gl_FragCoord.y - (uCanvasCoords.y + uCanvasCoords.w - r)) {
    //            discard;
    //        }
    //        if ((uCanvasCoords.x + uCanvasCoords.z) - gl_FragCoord.x < gl_FragCoord.y - (uCanvasCoords.y + uCanvasCoords.w - r)) {
    //            discard;
    //        }
    //        if (gl_FragCoord.x - uCanvasCoords.x < (uCanvasCoords.y + r) - gl_FragCoord.y) {
    //            discard;
    //        }
    //        if (gl_FragCoord.x - (uCanvasCoords.x + uCanvasCoords.z - r) > gl_FragCoord.y - uCanvasCoords.y) {
    //            discard;
    //        }
    //    }
    //    else if (type == 2) {
    //        if (gl_FragCoord.x < uCanvasCoords.x + r && gl_FragCoord.y > uCanvasCoords.y + uCanvasCoords.w - r && sq((uCanvasCoords.x + r) - gl_FragCoord.x) + sq(gl_FragCoord.y - (uCanvasCoords.y + uCanvasCoords.w - r)) > sq(r)) {
    //            discard;
    //        }
    //        if (gl_FragCoord.x > uCanvasCoords.x + uCanvasCoords.z - r && gl_FragCoord.y > uCanvasCoords.y + uCanvasCoords.w - r && sq(gl_FragCoord.x - (uCanvasCoords.x + uCanvasCoords.z - r)) + sq(gl_FragCoord.y - (uCanvasCoords.y + uCanvasCoords.w - r)) > sq(r)) {
    //            discard;
    //        }
    //        if (gl_FragCoord.x < uCanvasCoords.x + r && gl_FragCoord.y < uCanvasCoords.y + r && sq((uCanvasCoords.x + r) - gl_FragCoord.x) + sq((uCanvasCoords.y + r) - gl_FragCoord.y) > sq(r)) {
    //            discard;
    //        }
    //        if (gl_FragCoord.x > uCanvasCoords.x + uCanvasCoords.z - r && gl_FragCoord.y < uCanvasCoords.y + r && sq(gl_FragCoord.x - (uCanvasCoords.x + uCanvasCoords.z - r)) + sq((uCanvasCoords.y + r) - gl_FragCoord.y) > sq(r)) {
    //            discard;
    //        }
    //    }
    //}

    //if ((uCanvasCoords.x > gl_FragCoord.x || uCanvasCoords.x + uCanvasCoords.z < gl_FragCoord.x || uCanvasCoords.y > gl_FragCoord.y || uCanvasCoords.y + uCanvasCoords.w < gl_FragCoord.y) || ((type == 1 && r > 0) && ((gl_FragCoord.x - uCanvasCoords.x < gl_FragCoord.y - (uCanvasCoords.y + uCanvasCoords.w - r)) || ((uCanvasCoords.x + uCanvasCoords.z) - gl_FragCoord.x < gl_FragCoord.y - (uCanvasCoords.y + uCanvasCoords.w - r)) || (gl_FragCoord.x - uCanvasCoords.x < (uCanvasCoords.y + r) - gl_FragCoord.y) || (gl_FragCoord.x - (uCanvasCoords.x + uCanvasCoords.z - r) > gl_FragCoord.y - uCanvasCoords.y))) || ((type == 2 && r > 0) && ((gl_FragCoord.x < uCanvasCoords.x + r && gl_FragCoord.y > uCanvasCoords.y + uCanvasCoords.w - r && sq((uCanvasCoords.x + r) - gl_FragCoord.x) + sq(gl_FragCoord.y - (uCanvasCoords.y + uCanvasCoords.w - r)) > sq(r)) || (gl_FragCoord.x > uCanvasCoords.x + uCanvasCoords.z - r && gl_FragCoord.y > uCanvasCoords.y + uCanvasCoords.w - r && sq(gl_FragCoord.x - (uCanvasCoords.x + uCanvasCoords.z - r)) + sq(gl_FragCoord.y - (uCanvasCoords.y + uCanvasCoords.w - r)) > sq(r)) || (gl_FragCoord.x < uCanvasCoords.x + r && gl_FragCoord.y < uCanvasCoords.y + r && sq((uCanvasCoords.x + r) - gl_FragCoord.x) + sq((uCanvasCoords.y + r) - gl_FragCoord.y) > sq(r)) || (gl_FragCoord.x > uCanvasCoords.x + uCanvasCoords.z - r && gl_FragCoord.y < uCanvasCoords.y + r && sq(gl_FragCoord.x - (uCanvasCoords.x + uCanvasCoords.z - r)) + sq((uCanvasCoords.y + r) - gl_FragCoord.y) > sq(r))))) {
    //    discard;
    //}

    Material mat = uniforms.materials[int(matId)];

    //fancy calculations with material
    gPosition = pos;
    gNormal = normalize(normal);
    gNormal.rgb -= step(0.5, mat.normalTextureId) * texture(sampler2D(TEXTURES[int(mat.normalTextureId - 1)], SAMPLER), texCoord).rgb;
    gAlbedoSpec.a = mix(mat.metallic, texture(sampler2D(TEXTURES[int(mat.metallicRoughnessTextureId - 1)], SAMPLER), texCoord).a, min(mat.metallicRoughnessTextureId, 1.0));
    gAlbedoSpec.rgb = mix(mat.diffuse.rgb, texture(sampler2D(TEXTURES[int(mat.diffuseTextureId - 1)], SAMPLER), texCoord).rgb, min(mat.diffuseTextureId, 1.0));
}