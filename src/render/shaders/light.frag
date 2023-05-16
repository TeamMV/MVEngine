#version 450

out vec4 outColor;

in vec2 fTexCoord;

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

uniform float ambient = 0.5;

uniform vec3 viewPos;

uniform Light lights[MAX_NUM_LIGHTS]; //replaced in shader loader
uniform int numLights = MAX_NUM_LIGHTS;

void main() {
    vec3 FragPos = texture(gPosition, fTexCoord).rgb;
    vec3 Normal = texture(gNormals, fTexCoord).rgb;
    vec3 Albedo = texture(gAlbedoSpec, fTexCoord).rgb;
    float Specular = texture(gAlbedoSpec, fTexCoord).a;

    vec3 lighting = Albedo * ambient;
    vec3 viewDir = normalize(viewPos - FragPos);
    for (int i = 0; i < numLights; ++i) {
        float dist = length(lights[i].position - FragPos);
        //check for light volumes
        //if (dist < lights[i].radius) {
            // diffuse
            vec3 lightDir = normalize(lights[i].position - FragPos);
            vec3 diffuse = max(dot(Normal, lightDir), 0.0) * Albedo * lights[i].color;
            lighting += diffuse;
        //}
    }

    outColor = vec4(lighting, 1.0);
}