#version 450

precision highp float;

in vec2 fTexCoord;

out vec4 outColor;

uniform sampler2D tex;
uniform vec2 res;
uniform float time;

uniform float pixelSize;

void main() {
    outColor = vec4(1.0);
    return;
    vec4 color = texture(tex, fTexCoord);
    if (color.xyz == vec3(0.0)) {
        outColor = vec4(0.0);
        return;
    }
    outColor = vec4(1.0);
    return;
    vec2 d = 1.0 / res.xy;
    vec2 uv = (d.xy * float(pixelSize)) * floor(fTexCoord.xy * res.xy / float(pixelSize));

    outColor = vec4(0);

    for (int i = 0; i < pixelSize; i++) {
        for (int j = 0; j < pixelSize; j++) {
            outColor += texture(tex, uv.xy + vec2(d.x * float(i), d.y * float(j)));
        }
    }

    outColor /= pow(float(pixelSize), 2.0);
}