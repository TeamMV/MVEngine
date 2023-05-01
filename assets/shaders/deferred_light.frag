#version 450

out vec4 outColor;

in vec2 fTexCoord;

struct Light {
    vec3 position;
    vec3 direcetion;
    vec4 color;
    float attenuation;
    float cutoff;
    float radius;
};

uniform sampler2D gAlbedoSpec;
uniform sampler2D gNormals;
uniform sampler2D gPosition;

uniform float ambient = 0.1;

uniform vec3 viewPos;

uniform Light lights[NUM_LIGHTS]; //replaced in shader loader

void main() {
    vec3 FragPos = texture(gPosition, fTexCoord).rgb;
    vec3 Normal = texture(gNormals, fTexCoord).rgb;
    vec3 Albedo = texture(gAlbedoSpec, fTexCoord).rgb;
    float Specular = texture(gAlbedoSpec, fTexCoord).a;

    vec3 lighting = Albedo * ambient;
    vec3 viewDir = normalize(viewPos - FragPos);
    for (int i = 0; i < NUM_LIGHTS; ++i) {
        float dist = length(lights[i].position - FragPos);
        //check for light volumes
        if (dist < lights[i].radius) {
            // diffuse
            vec3 lightDir = normalize(lights[i].position - FragPos);
            vec3 diffuse = max(dot(Normal, lightDir), 0.0) * Albedo * lights[i].color;
            lighting += diffuse;
        }
    }

    outColor = vec4(lighting, 1.0);
}