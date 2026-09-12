#version 440

layout(std140, binding = 0) uniform buf {
    mat4 qt_Matrix;
    float qt_Opacity;
    float time;
    vec2 resolution;
    vec4 accentColor;
    vec4 coreColor;
} ubuf;

layout(location = 0) in vec2 qt_TexCoord0;
layout(location = 0) out vec4 fragColor;

const vec2 RR = vec2(0.640, 0.235);
const float TALL = 1.18;

vec2 orbit(float a) {
    return vec2(cos(a) * RR.x, sin(a) * RR.y + sin(ubuf.time * 1.35) * 0.08);
}

vec2 heading(float a) {
    return normalize(vec2(-sin(a) * RR.x, cos(a) * RR.y));
}

float speed(float a) {
    return length(vec2(-sin(a) * RR.x, cos(a) * RR.y)) / RR.x;
}

float blob(vec2 p, vec2 c, float r, vec2 tang, float stretch, float tall) {
    vec2 d = p - c;
    d.y /= tall;
    vec2 n = vec2(-tang.y, tang.x);
    vec2 dl = vec2(dot(d, tang) / stretch, dot(d, n) * stretch);
    return (r * r) / (dot(dl, dl) + 1e-5);
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
    vec2 p = qt_TexCoord0 - 0.5;
    p.x *= aspect;
    p.y = -p.y;

    float a = ubuf.time * 4.6;

    float v = 0.0;
    for (int i = 0; i < 8; i++) {
        float fi = float(i);
        float ai = a - fi * 0.160;
        float near = (1.0 - sin(ai)) * 0.5;
        float taper = exp(-fi * 0.46);
        float scale = mix(0.80, 1.15, near);
        float fade = mix(0.40, 1.0, near);
        float stretch = 1.0 + (speed(ai) - 0.55) * 0.62;

        vec2 blobCenter = orbit(ai);
        blobCenter.y += sin(ai * 2.4 + ubuf.time * 3.0) * 0.035 * (1.0 - taper);

        v += blob(p, blobCenter, 0.180 * scale * taper, heading(ai), stretch, TALL) * fade * taper;
    }

    float near = (1.0 - sin(a)) * 0.5;
    float hs = mix(0.80, 1.15, near);
    vec2 head = orbit(a);
    vec2 hd = heading(a);
    vec2 swept = head - hd * (0.030 * speed(a));
    float armDim = mix(0.55, 1.0, near);

    v += limb(p, swept + vec2(-0.105, -0.020) * hs,
                 swept + vec2(-0.178, -0.086) * hs, 0.046 * hs) * armDim;
    v += limb(p, swept + vec2( 0.105, -0.020) * hs,
                 swept + vec2( 0.178, -0.086) * hs, 0.046 * hs) * armDim;

    float aa = clamp(fwidth(v), 0.02, 0.9);
    float mask = smoothstep(1.0 - aa, 1.0 + aa, v);

    vec3 col = mix(ubuf.accentColor.rgb, ubuf.coreColor.rgb, smoothstep(1.15, 3.20, v));

    vec2 face = head + hd * 0.042;
    vec2 eyeOffset = vec2(0.075, 0.035) * hs;

    float blink = step(0.96, sin(ubuf.time * 4.5));
    float farSquish = mix(3.5, 1.0, smoothstep(0.1, 0.6, near));
    float ySquish = max(farSquish, blink * 8.0);

    vec2 leftEye = p - face - vec2(-eyeOffset.x, eyeOffset.y);
    leftEye.y *= ySquish;
    vec2 rightEye = p - face - vec2(eyeOffset.x, eyeOffset.y);
    rightEye.y *= ySquish;

    float de = min(length(leftEye), length(rightEye));
    float eyeR = 0.048 * hs;
    float eyeAlpha = (1.0 - smoothstep(eyeR * 0.5, eyeR, de)) * mix(0.3, 1.0, near);
    col = mix(col, vec3(0.15, 0.12, 0.16), eyeAlpha);

    vec2 leftBlush = p - face - vec2(-eyeOffset.x * 1.3, eyeOffset.y - 0.045 * hs);
    vec2 rightBlush = p - face - vec2(eyeOffset.x * 1.3, eyeOffset.y - 0.045 * hs);
    float dBlush = min(length(leftBlush), length(rightBlush));
    float blushAlpha = (1.0 - smoothstep(0.0, 0.07 * hs, dBlush)) * smoothstep(0.3, 1.0, near);
    col = mix(col, vec3(0.9, 0.35, 0.45), blushAlpha * 0.45);

    fragColor = vec4(col, 1.0) * mask * ubuf.qt_Opacity;
}
