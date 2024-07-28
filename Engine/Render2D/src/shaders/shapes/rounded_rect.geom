#version 460 core

#extension GL_EXT_scalar_block_layout : enable

layout (points) in;
layout (triangle_strip, max_vertices = 252) out;

const float M_PI = 3.1415926535897F;  // PI
const float M_PI_OVER_2 = M_PI / 2.0f;

layout(location = 0) in DataIn
{
    vec2 scale;
    vec3 rotation;
    float border_radius;
    int smoothness;
    vec4 texCoords; // x, y, width, height,
    vec4 color;
    mat4 view;
    mat4 proj;
} gs_in[];

layout(location = 0) out vec2 outTexCoord;
layout(location = 1) out vec4 outColor;

const int MAX_SMOOTHNESS = 20;

mat4 translate(vec3 translation) {
    return mat4(
    vec4(1.0, 0.0, 0.0, 0.0),
    vec4(0.0, 1.0, 0.0, 0.0),
    vec4(0.0, 0.0, 1.0, 0.0),
    vec4(translation, 1.0)
    );
}

void EmitCurve(vec2 curve[MAX_SMOOTHNESS + 1], vec2 center, float xMult, float yMult)
{
    mat4 translationMatrix = translate(gl_in[0].gl_Position.xyz + vec3(center, 0.0f));

    mat4 modelMatrix = translationMatrix;

    for (int i = 1; i <= gs_in[0].smoothness; i++)
    {
        gl_Position = gs_in[0].proj * gs_in[0].view * modelMatrix * vec4(0.0f, 0.0f, 1.0f, 1.0f);
        EmitVertex();
        gl_Position = gs_in[0].proj * gs_in[0].view * modelMatrix * vec4(curve[i-1].x * xMult, curve[i-1].y * yMult, 1.0f, 1.0f);
        EmitVertex();
        gl_Position = gs_in[0].proj * gs_in[0].view * modelMatrix * vec4(curve[i].x * xMult, curve[i].y * yMult, 1.0f, 1.0f);
        EmitVertex();
        EndPrimitive();
    }
}

void EmitRect(vec2 position, vec2 scale)
{
    mat4 translationMatrix = translate(gl_in[0].gl_Position.xyz + vec3(position, 0.0f));

    mat4 scaleMatrix = mat4(
        scale.x, 0.0, 0.0, 0.0,
        0.0, scale.y, 0.0, 0.0,
        0.0, 0.0, 1.0f, 0.0,
        0.0, 0.0, 0.0, 1.0
    );

    mat4 modelMatrix = translationMatrix * scaleMatrix;

    gl_Position = gs_in[0].proj * gs_in[0].view * modelMatrix * vec4(0.0f, 0.0f, 1.0f, 1.0f);
    EmitVertex();
    gl_Position = gs_in[0].proj * gs_in[0].view * modelMatrix * vec4(1.0f, 0.0f, 1.0f, 1.0f);
    EmitVertex();
    gl_Position = gs_in[0].proj * gs_in[0].view * modelMatrix * vec4(0.0f, 1.0f, 1.0f, 1.0f);
    EmitVertex();
    gl_Position = gs_in[0].proj * gs_in[0].view * modelMatrix * vec4(1.0f, 1.0f, 1.0f, 1.0f);
    EmitVertex();
    EndPrimitive();
}

void main()
{
    outColor = vec4(1.0);
    int smoothness = gs_in[0].smoothness;
    vec2 scale = gs_in[0].scale;
    float angle = M_PI_OVER_2 / float(smoothness);
    float x_radius = clamp(gs_in[0].border_radius, 0, gs_in[0].scale.x / 2.0);
    float y_radius = clamp(gs_in[0].border_radius, 0, gs_in[0].scale.y / 2.0);

    vec2 curve[MAX_SMOOTHNESS + 1];

    curve[0] = vec2(x_radius, 0.0);
    curve[smoothness] = vec2(0.0, y_radius);

    for (int i = 1; i < smoothness; i++) {
        float step_angle = angle * float(i);
        curve[i].x = x_radius * cos(step_angle);
        curve[i].y = y_radius * sin(step_angle);
    }

    // Emit curves
    EmitCurve(curve, vec2(x_radius, y_radius), -1.0f, -1.0f);
    EmitCurve(curve, vec2(x_radius, scale.y - y_radius), -1.0f, 1.0f);
    EmitCurve(curve, vec2(scale.x - x_radius, y_radius), 1.0f, -1.0f);
    EmitCurve(curve, vec2(scale.x - x_radius, scale.y - y_radius), 1.0f, 1.0f);

    // Emit inner rectangles
    EmitRect(vec2(x_radius, 0), vec2(scale.x - x_radius * 2, y_radius));
    EmitRect(vec2(0, y_radius), vec2(scale.x, scale.y - y_radius * 2));
    EmitRect(vec2(x_radius, scale.y - y_radius), vec2(scale.x - x_radius * 2, y_radius));
}