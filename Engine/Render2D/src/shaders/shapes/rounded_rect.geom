#version 460 core

#extension GL_EXT_scalar_block_layout : enable

layout (points) in;
layout (triangle_strip, max_vertices = 252) out;

const float M_PI = 3.1415926535897F;  // PI
const float M_PI_OVER_2 = M_PI / 2.0f;

layout(location = 0) in DataIn
{
    mat4 view;
    mat4 proj;
    vec4 scale;
    vec4 rotation;
    vec4 texCoords; // x, y, width, height,
    vec4 color;
    vec2 screenSize;
    int texId;
    float border_radius;
    int smoothness;
    float blending;
} gs_in[];

layout(location = 0) out vec2 outTexCoord;
layout(location = 1) out vec4 outColor;
layout(location = 2) out int outTexId;
layout(location = 3) out float outBlending;

const int MAX_SMOOTHNESS = 20;

mat4 translate(vec3 translation) {
    return mat4(
    vec4(1.0, 0.0, 0.0, 0.0),
    vec4(0.0, 1.0, 0.0, 0.0),
    vec4(0.0, 0.0, 1.0, 0.0),
    vec4(translation, 1.0)
    );
}

void CalculateTexCoords(vec4 rectangle)
{
    outTexCoord = ((gl_Position.xy - rectangle.xy) / rectangle.zw) * gs_in[0].texCoords.zw + gs_in[0].texCoords.xy;
}

void EmitCurve(vec2 curve[MAX_SMOOTHNESS + 1], vec2 center, float xMult, float yMult, vec4 rectangle, mat4 rotationMat)
{
    mat4 translationMatrix = translate(gl_in[0].gl_Position.xyz + vec3(center, 0.0f));

    mat4 modelMatrix = translationMatrix;

    for (int i = 1; i <= gs_in[0].smoothness; i++)
    {
        gl_Position = gs_in[0].proj * gs_in[0].view * modelMatrix * vec4(0.0f, 0.0f, 0.0f, 1.0f);
        CalculateTexCoords(rectangle);
        gl_Position = rotationMat * gl_Position;
        EmitVertex();
        gl_Position = gs_in[0].proj * gs_in[0].view * modelMatrix * vec4(curve[i-1].x * xMult, curve[i-1].y * yMult, 0.0f, 1.0f);
        CalculateTexCoords(rectangle);
        gl_Position = rotationMat * gl_Position;
        EmitVertex();
        gl_Position = gs_in[0].proj * gs_in[0].view * modelMatrix * vec4(curve[i].x * xMult, curve[i].y * yMult, 0.0f, 1.0f);
        CalculateTexCoords(rectangle);
        gl_Position = rotationMat * gl_Position;
        EmitVertex();
        EndPrimitive();
    }
}

void EmitRect(vec2 position, vec2 scale, vec4 rectangle, mat4 rotationMat)
{
    mat4 translationMatrix = translate(gl_in[0].gl_Position.xyz + vec3(position, 0.0f));

    mat4 scaleMatrix = mat4(
        scale.x, 0.0, 0.0, 0.0,
        0.0, scale.y, 0.0, 0.0,
        0.0, 0.0, 1.0f, 0.0,
        0.0, 0.0, 0.0, 1.0
    );

    mat4 modelMatrix = translationMatrix * scaleMatrix;

    gl_Position = gs_in[0].proj * gs_in[0].view * modelMatrix * vec4(0.0f, 0.0f, 0.0f, 1.0f);
    CalculateTexCoords(rectangle);
    gl_Position = rotationMat * gl_Position;
    EmitVertex();
    gl_Position = gs_in[0].proj * gs_in[0].view * modelMatrix * vec4(1.0f, 0.0f, 0.0f, 1.0f);
    CalculateTexCoords(rectangle);
    gl_Position = rotationMat * gl_Position;
    EmitVertex();
    gl_Position = gs_in[0].proj * gs_in[0].view * modelMatrix * vec4(0.0f, 1.0f, 0.0f, 1.0f);
    CalculateTexCoords(rectangle);
    gl_Position = rotationMat * gl_Position;
    EmitVertex();
    gl_Position = gs_in[0].proj * gs_in[0].view * modelMatrix * vec4(1.0f, 1.0f, 0.0f, 1.0f);
    CalculateTexCoords(rectangle);
    gl_Position = rotationMat * gl_Position;
    EmitVertex();
    EndPrimitive();
}

void main()
{
    outColor = gs_in[0].color;
    outTexId = gs_in[0].texId;
    outBlending = gs_in[0].blending;
    int smoothness = gs_in[0].smoothness;
    vec2 scale = gs_in[0].scale.xy;
    float angle = M_PI_OVER_2 / float(smoothness);
    float x_radius = clamp(gs_in[0].border_radius, 0, gs_in[0].scale.x / 2.0);
    float y_radius = clamp(gs_in[0].border_radius, 0, gs_in[0].scale.y / 2.0);

    vec2 curve[MAX_SMOOTHNESS + 1];

    vec2 rectangleScale = (scale * 2.0f) / gs_in[0].screenSize;
    vec4 screenSpacePosition = gs_in[0].proj * gs_in[0].view * vec4(gl_in[0].gl_Position.xyz, 1.0f);
    vec4 rectangle = vec4(screenSpacePosition.xy, rectangleScale);

    curve[0] = vec2(x_radius, 0.0);
    curve[smoothness] = vec2(0.0, y_radius);

    for (int i = 1; i < smoothness; i++) {
        float step_angle = angle * float(i);
        curve[i].x = x_radius * cos(step_angle);
        curve[i].y = y_radius * sin(step_angle);
    }

    vec3 rotation = gs_in[0].rotation.xyz;

    mat4 rotateX = mat4(
        1.0, 0.0, 0.0, 0.0,
        0.0, cos(rotation.x), -sin(rotation.x), 0.0,
        0.0, sin(rotation.x), cos(rotation.x), 0.0,
        0.0, 0.0, 0.0, 1.0
    );

    mat4 rotateY = mat4(
        cos(rotation.y), 0.0, sin(rotation.y), 0.0,
        0.0, 1.0, 0.0, 0.0,
        -sin(rotation.y), 0.0, cos(rotation.y), 0.0,
        0.0, 0.0, 0.0, 1.0
    );

    mat4 rotateZ = mat4(
        cos(rotation.z), -sin(rotation.z), 0.0, 0.0,
        sin(rotation.z), cos(rotation.z), 0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        0.0, 0.0, 0.0, 1.0
    );

    mat4 translationMatrix = translate(gl_in[0].gl_Position.xyz + vec3(vec2(x_radius, y_radius), 0.0f));

    mat4 modelMatrix = translationMatrix;

    mat4 rotateMatrix = rotateX * rotateY * rotateZ;

    // Emit curves
    EmitCurve(curve, vec2(x_radius, y_radius), -1.0f, -1.0f, rectangle, rotateMatrix);
    EmitCurve(curve, vec2(x_radius, scale.y - y_radius), -1.0f, 1.0f, rectangle, rotateMatrix);
    EmitCurve(curve, vec2(scale.x - x_radius, y_radius), 1.0f, -1.0f, rectangle, rotateMatrix);
    EmitCurve(curve, vec2(scale.x - x_radius, scale.y - y_radius), 1.0f, 1.0f, rectangle, rotateMatrix);

    // Emit inner rectangles
    EmitRect(vec2(x_radius, 0), vec2(scale.x - x_radius * 2, y_radius), rectangle, rotateMatrix);
    EmitRect(vec2(0, y_radius), vec2(scale.x, scale.y - y_radius * 2), rectangle, rotateMatrix);
    EmitRect(vec2(x_radius, scale.y - y_radius), vec2(scale.x - x_radius * 2, y_radius), rectangle, rotateMatrix);
}