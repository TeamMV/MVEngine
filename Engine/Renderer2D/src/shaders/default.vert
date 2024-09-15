#version 450

precision highp float;

struct Vertex {
    vec3 position;
    vec3 rotation;
    vec3 origin;
    CanvasTransform transform;
    vec4 color;
    float texture_id;
    float use_camera;
};

struct CanvasTransform {
    vec3 translation;
    vec3 rotation;
    vec2 scale;
    vec3 origin;
};

struct CameraBuffer {
    mat4 projection;
    mat4 view;
    mat4 world;
    vec2 resolution;
};

uniform mat4 uProjection;
uniform mat4 uView;

uniform float uResX;
uniform float uResY;

void main() {

}