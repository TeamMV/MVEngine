#version 450

out vec2 fTexCoord;

vec2 positions[4] = vec2[](
    vec2(-1.0,-1.0),
    vec2(-1.0,1.0),
    vec2(1.0,-1.0),
    vec2(1.0,1.0)
);

void main() {
    fTexCoord = positions[gl_VertexID];
    gl_Position = vec4(positions[gl_VertexID], 0.0, 1.0);
}