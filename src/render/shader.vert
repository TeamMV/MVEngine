#version 450

const vec2 pos[3] = vec2[](
    vec2(-0.5, -0.5),
    vec2(0.0, 0.5),
    vec2(0.5, -0.5)
);

const vec3 col[3] = vec3[](
    vec3(1.0, 0.0, 0.0),
    vec3(0.0, 1.0, 0.0),
    vec3(0.0, 0.0, 1.0)
);

out vec4 fCol;

void main() {
    gl_Position = vec4(pos[gl_VertexIndex], 0.0, 1.0);
    fCol = vec4(col[gl_VertexIndex], 1.0);
}