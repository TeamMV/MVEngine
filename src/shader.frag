#version 420

layout(location = 0) in vec3 fColor;

layout(location = 0) out vec4 outColor;

void main() {
    fColor = pow(fColor, 1.0f / 2.2f);
    outColor = vec4(fColor, 1.0f);
}