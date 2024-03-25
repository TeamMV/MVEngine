#version 420

layout(location = 0) in vec4 col;

layout(location = 0) out vec4 outColor;

void main() {
    vec3 color = pow(col.rgb, vec3(1.0f / 2.2f));
    outColor = vec4(color, col.a);
    //outColor = col;
}