#version 450

layout (location = 0) in vec2 fUv;

layout(location = 0) out vec4 outColor;

uniform sampler2D COLOR;
uniform sampler2D DEPTH;

void main() {
    vec4 color = texture(COLOR, fUv);

    float grayscale = dot(color.rgb, vec3(0.299, 0.587, 0.114));

    outColor = vec4(vec3(grayscale), 1.0);
}
