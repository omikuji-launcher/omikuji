#version 440

layout(std140, binding = 0) uniform buf {
    mat4 qt_Matrix;
    float qt_Opacity;
    float time;
    float lookUp;
    float squeeze;
    float wobble;
    vec2 resolution;
    vec4 accentColor;
    vec4 coreColor;
} ubuf;

layout(location = 0) in vec2 qt_TexCoord0;
layout(location = 0) out vec4 fragColor;

const float TALL = 1.18;
const vec2 EYE = vec2(0.075, 0.035);
const float EYE_R = 0.048;

float blob(vec2 p, vec2 c, float r) {
    vec2 d = p - c;
    d.y /= TALL;
    return (r * r) / (dot(d, d) + 1e-5);
}

float limb(vec2 p, vec2 a, vec2 b, float r) {
    vec2 pa = p - a;
    vec2 ba = b - a;
    float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    vec2 d = pa - ba * h;
    return (r * r) / (dot(d, d) + 1e-5);
}

void main() {
    float aspect = ubuf.resolution.x / max(ubuf.resolution.y, 1.0);
    vec2 uv = vec2((qt_TexCoord0.x - 0.5) * aspect, 0.5 - qt_TexCoord0.y);

    float t = ubuf.time;
    vec2 p = uv * 0.62;

    float w = clamp(ubuf.wobble, 0.0, 1.0);
    p.x += sin(p.y * 21.0 + t * 13.0) * 0.0075 * w;
    p.y += sin(p.x * 19.0 - t * 11.0) * 0.0060 * w;

    float v = blob(p, vec2(0.0, 0.006), 0.185);

    vec2 m = vec2(abs(p.x), p.y);
    v += limb(m, vec2(0.105, -0.020), vec2(0.150, -0.078), 0.036);

    float aa = clamp(fwidth(v), 0.02, 0.9);
    float mask = smoothstep(1.0 - aa, 1.0 + aa, v);

    vec3 col = mix(ubuf.accentColor.rgb, ubuf.coreColor.rgb, smoothstep(1.15, 3.20, v));

    // before the eyes on purpose, or it tints his pupils pink
    vec2 blush = m - vec2(EYE.x * 1.30, EYE.y - 0.062);
    col = mix(col, vec3(0.90, 0.35, 0.45),
              (1.0 - smoothstep(0.0, 0.062, length(blush))) * 0.45);

    float squish = mix(1.0, 8.0, clamp(ubuf.squeeze, 0.0, 1.0));
    vec2 eye = m - vec2(EYE.x, EYE.y + ubuf.lookUp * 0.024);
    eye.y *= squish;
    float eyeD = length(eye);
    float eyeAA = fwidth(eyeD) + 1e-4;
    col = mix(col, vec3(0.15, 0.12, 0.16),
              1.0 - smoothstep(EYE_R - eyeAA, EYE_R + eyeAA, eyeD));

    fragColor = vec4(col, 1.0) * mask * ubuf.qt_Opacity;
}
