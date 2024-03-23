#version 460

layout (local_size_x = 8, local_size_y = 8) in;

layout(set = 0, binding = 0) uniform sampler2D uInput;
layout(set = 0, binding = 1, rgba8) uniform image2D uOutput;

struct TonemapInfo
{
    float Contrast;
    float Saturation;
    float Exposure;
    float Brightness;
    float Vignette;
    float Gamma;
    float Temperature;
    float Tint;
    vec4 ColorFilter;

    ivec2 AberrationOffsets[3];
    float AberrationVignette;

    float WhitePointReinhard;
};

layout(push_constant) uniform PushConstants
{
    TonemapInfo pTonemapInfo;
};

const mat3 AcesInputMatrix =
{
vec3(0.59719f, 0.35458f, 0.04823f),
vec3(0.07600f, 0.90834f, 0.01566f),
vec3(0.02840f, 0.13383f, 0.83777f)
};

const mat3 AcesOutputMatrix =
{
vec3( 1.60475f, -0.53108f, -0.07367f),
vec3(-0.10208f,  1.10813f, -0.00605f),
vec3(-0.00327f, -0.07276f,  1.07602f)
};

float CalculateLuminance(vec3 rgb)
{
    return 0.212671f * rgb.r + 0.715160f * rgb.g + 0.072169f * rgb.b;
}

vec3 RttAndOdtFit(vec3 v)
{
    vec3 a = v * (v + 0.0245786f) - 0.000090537f;
    vec3 b = v * (0.983729f * v + 0.4329510f) + 0.238081f;
    return a / b;
}

vec3 Uncharted2TonemapPartial(vec3 x)
{
    float A = 0.15f;
    float B = 0.50f;
    float C = 0.10f;
    float D = 0.20f;
    float E = 0.02f;
    float F = 0.30f;
    return ((x*(A*x+C*B)+D*E)/(x*(A*x+B)+D*F))-E/F;
}

vec3 Uncharted2(vec3 v)
{
    float exposure_bias = 2.0f;
    vec3 curr = Uncharted2TonemapPartial(v * exposure_bias);

    vec3 W = vec3(11.2f);
    vec3 white_scale = vec3(1.0f) / Uncharted2TonemapPartial(W);
    return clamp(curr * white_scale, vec3(0.0f), vec3(1.0f));
}

vec3 HillAces(vec3 color)
{
    color = color * AcesInputMatrix;
    color = RttAndOdtFit(color);
    return color * AcesOutputMatrix;
}

// Faster approximation of Hill Aces
vec3 NarkowiczAces(vec3 color)
{
    color = (color*(2.51f*color+0.03f))/(color*(2.43f*color+0.59f)+0.14f);
    return color;
}

vec3 Filmic(vec3 color)
{
    vec3 temp   = max(vec3(0.0F), color - vec3(0.004F));
    vec3 result = (temp * (vec3(6.2F) * temp + vec3(0.5F))) / (temp * (vec3(6.2F) * temp + vec3(1.7F)) + vec3(0.06F));
    return clamp(result, vec3(0.0f), vec3(1.0f));
}

vec3 WhiteBalance(vec3 col, float temp, float tint)
{
    float t1 = temp * 10.0f / 6.0f;
    float t2 = tint * 10.0f / 6.0f;

    float x = 0.31271 - t1 * (t1 < 0 ? 0.1 : 0.05);
    float standardIlluminantY = 2.87 * x - 3 * x * x - 0.27509507;
    float y = standardIlluminantY + t2 * 0.05;

    vec3 w1 = vec3(0.949237, 1.03542, 1.08728);

    float Y = 1;
    float X = Y * x / y;
    float Z = Y * (1 - x - y) / y;
    float L = 0.7328 * X + 0.4296 * Y - 0.1624 * Z;
    float M = -0.7036 * X + 1.6975 * Y + 0.0061 * Z;
    float S = 0.0030 * X + 0.0136 * Y + 0.9834 * Z;
    vec3 w2 = vec3(L, M, S);

    vec3 balance = vec3(w1.x / w2.x, w1.y / w2.y, w1.z / w2.z);

    mat3 LIN_2_LMS_MAT = mat3(
    vec3(3.90405e-1, 5.49941e-1, 8.92632e-3),
    vec3(7.08416e-2, 9.63172e-1, 1.35775e-3),
    vec3(2.31082e-2, 1.28021e-1, 9.36245e-1)
    );

    mat3 LMS_2_LIN_MAT = mat3(
    2.85847e+0, -1.62879e+0, -2.48910e-2,
    -2.10182e-1,  1.15820e+0,  3.24281e-4,
    -4.18120e-2, -1.18169e-1,  1.06867e+0
    );

    vec3 lms = col * LIN_2_LMS_MAT;
    lms *= balance;
    return lms * LMS_2_LIN_MAT;
}

vec3 ReinhardExtended(vec3 color, float whitePoint)
{
    float Lin = CalculateLuminance(color);

    float Lout = (Lin * (1.0 + Lin / (whitePoint * whitePoint))) / (1.0 + Lin);

    vec3 Cout = color / Lin * Lout;

    return clamp(Cout, 0.0f, 1.0f);
}

void main()
{
    if(gl_GlobalInvocationID.xy != clamp(gl_GlobalInvocationID.xy, vec2(0.0F), imageSize(uOutput)))
    return;

    vec2 pixelCoord = vec2(gl_GlobalInvocationID.xy) + vec2(0.5f);
    vec2 textureSize = vec2(textureSize(uInput, 0));
    vec2 texCoord = pixelCoord / textureSize;
    vec2 centeredTexCoord = ((texCoord - vec2(0.5f)) * vec2(2.0f));

    #ifdef USE_CHROMATIC_ABERRATION
    // Chromatic aberration
    int aberrationVignette = int(round(clamp(length(centeredTexCoord) * pTonemapInfo.AberrationVignette * 3, 0, 3)));

    ivec2 uvRed     = clamp(ivec2(gl_GlobalInvocationID.xy) + ivec2(pTonemapInfo.AberrationOffsets[0].x * aberrationVignette, pTonemapInfo.AberrationOffsets[0].y * aberrationVignette), ivec2(0, 0), ivec2(textureSize) - 1);
    ivec2 uvGreen   = clamp(ivec2(gl_GlobalInvocationID.xy) + ivec2(pTonemapInfo.AberrationOffsets[1].x * aberrationVignette, pTonemapInfo.AberrationOffsets[1].y * aberrationVignette), ivec2(0, 0), ivec2(textureSize) - 1);
    ivec2 uvBlue    = clamp(ivec2(gl_GlobalInvocationID.xy) + ivec2(pTonemapInfo.AberrationOffsets[2].x * aberrationVignette, pTonemapInfo.AberrationOffsets[2].y * aberrationVignette), ivec2(0, 0), ivec2(textureSize) - 1);

    vec3 hdrColor;
    hdrColor.r = texelFetch(uInput, uvRed, 0).r;
    hdrColor.g = texelFetch(uInput, uvGreen, 0).g;
    hdrColor.b = texelFetch(uInput, uvBlue, 0).b;
    #else
    vec3 hdrColor = texelFetch(uInput, ivec2(gl_GlobalInvocationID.xy), 0).rgb;
    #endif

    // Exposure
    hdrColor *= pTonemapInfo.Exposure;
    vec3 mapped = hdrColor;
    mapped = max(vec3(0.0f), mapped);

    // White Balancing
    mapped = WhiteBalance(mapped, pTonemapInfo.Temperature, pTonemapInfo.Tint);
    mapped = max(vec3(0.0f), mapped);

    // contrast & brightness
    mapped = pTonemapInfo.Contrast * (mapped - 0.5f) + 0.5f + pTonemapInfo.Brightness;
    mapped = max(vec3(0.0f), mapped);

    // Color Filter
    mapped *= pTonemapInfo.ColorFilter.xyz;
    mapped = max(vec3(0.0f), mapped);

    // saturation
    vec3 i = vec3(dot(mapped, vec3(0.299f, 0.587f, 0.114f)));
    mapped = mix(i, mapped, pTonemapInfo.Saturation);
    mapped = max(vec3(0.0f), mapped);

    // vignette
    float vignette = clamp(1.0f - length(centeredTexCoord) * pTonemapInfo.Vignette, 0.0f, 1.0f);
    mapped *= vignette;

    #ifdef USE_REINHARD_EXTENDED
    mapped = ReinhardExtended(mapped, pTonemapInfo.WhitePointReinhard);
    #endif

    #ifdef USE_FILMIC
    mapped = Filmic(mapped), vec3(0.0f), vec3(1.0f);
    #endif

    #ifdef USE_ACES_HILL
    mapped = HillAces(mapped);
    #endif

    #ifdef USE_ACES_NARKOWICZ
    mapped = NarkowiczAces(mapped);
    #endif

    #ifdef USE_UNCHARTED
    mapped = Uncharted2(mapped);
    #endif

    // Simple Exposure mapping
    #ifdef USE_EXPOSURE_MAPPING
    mapped = clamp(vec3(1.0) - exp(-mapped), vec3(0.0f), vec3(1.0f));
    #endif

    mapped = pow(mapped, vec3(pTonemapInfo.Gamma));

    imageStore(uOutput, ivec2(gl_GlobalInvocationID.xy), vec4(mapped, 1.0f));
}