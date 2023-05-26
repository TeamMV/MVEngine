#version 450

const int MAX_NUM_LIGHTS = 1;

layout(location = 0) out vec4 outColor;

layout(location = 0) in vec2 fTexCoord;

struct Light {
    vec3 position;
    vec3 direcetion;
    vec3 color;
    float attenuation;
    float cutoff; //if > 0 -> spotlight
    float radius; //if 0 -> direction light
};

layout(set = 0, binding = 0) uniform texture2D gAlbedoSpec;
layout(set = 0, binding = 1) uniform texture2D gNormals;
layout(set = 0, binding = 2) uniform texture2D gPosition;
layout(set = 0, binding = 3) uniform sampler SAMPLER;

layout(set = 0, binding = 4) uniform UNIFORMS {
   float ambient;
   vec3 viewPos;
   Light lights[MAX_NUM_LIGHTS]; //replaced in shader loader
   int numLights;
} uniforms;

void main() {
    vec3 FragPos = texture(sampler2D(gPosition, SAMPLER), fTexCoord).rgb;
    vec3 Normal = texture(sampler2D(gNormals, SAMPLER), fTexCoord).rgb;
    vec3 Albedo = texture(sampler2D(gAlbedoSpec, SAMPLER), fTexCoord).rgb;
    float Specular = texture(sampler2D(gAlbedoSpec, SAMPLER), fTexCoord).a;

    vec3 lighting = Albedo * uniforms.ambient;
    vec3 viewDir = normalize(uniforms.viewPos - FragPos);
    for (int i = 0; i < uniforms.numLights; ++i) {
        float dist = length(uniforms.lights[i].position - FragPos);
        //check for light volumes
        //if (dist < lights[i].radius) {
            // diffuse
            vec3 lightDir = normalize(uniforms.lights[i].position - FragPos);
            vec3 diffuse = max(dot(Normal, lightDir), 0.0) * Albedo * uniforms.lights[i].color;
            lighting += diffuse;
        //}
    }

    outColor = vec4(lighting, 1.0);
}