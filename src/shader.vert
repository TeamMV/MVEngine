#version 420

layout(location = 0) out vec3 fColor;

vec3 colors[] = vec3[](
    vec3(1.0),
    vec3(1.0),
    vec3(1.0)
);

vec2 pos[] = vec2[](
    vec2(1.0, 0.0),
    vec2(-1.0, 0.0),
    vec2(0.0, 0.5)
);

void main() {
    gl_Position = vec4(pos[gl_VertexIndex], 0.0, 1.0);
    fColor = colors[gl_VertexIndex];
}