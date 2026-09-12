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

float blob(vec2 p, vec2 c, float r) {
    vec2 d = p - c;
    return (r * r) / (dot(d, d) + 1e-5);
}

void main() {
    float aspect = ubuf.resolution.x / max(ubuf.resolution.y, 1.0);
    vec2 p = vec2((qt_TexCoord0.x - 0.5) * aspect, 1.0 - qt_TexCoord0.y);

    float t = ubuf.time * 1.30;

    float h = smoothstep(0.06, 0.78, p.y);
    vec2 q = p;
    q.x -= (sin(p.y * 11.0 + t * 2.63) * 0.011
          + sin(p.y * 17.3 - t * 1.71) * 0.006
          + sin(p.y * 6.70 + t * 3.29) * 0.008) * h;
    q.y -= sin(p.x * 13.0 + t * 2.11) * 0.007 * h;

    float v = 0.0;
    v += blob(q, vec2(0.0, 0.300), 0.150 * (1.0 + sin(t * 2.30) * 0.035));
    v += blob(q, vec2(0.0, 0.455), 0.082 * (1.0 + sin(t * 3.10 + 1.7) * 0.070));
    v += blob(q, vec2(0.0, 0.565), 0.042 * (1.0 + sin(t * 3.90 + 0.4) * 0.110));
    v += blob(q, vec2(0.0, 0.645), 0.020 * (1.0 + sin(t * 4.70 + 2.9) * 0.160));

    for (int i = 0; i < 2; i++) {
        float fi = float(i);
        float cyc = fract(t * 0.22 + fi * 0.5);
        float dy = 0.600 + cyc * 0.360;
        float dx = sin(t * 1.9 + fi * 2.4) * 0.026 * cyc;
        float dr = 0.030 * (1.0 - cyc) * smoothstep(0.0, 0.15, cyc);
        v += blob(q, vec2(dx, dy), dr);
    }

    float aa = clamp(fwidth(v), 0.02, 0.9);
    float a = smoothstep(1.0 - aa, 1.0 + aa, v);

    vec3 col = mix(ubuf.accentColor.rgb, ubuf.coreColor.rgb, smoothstep(1.15, 3.00, v));
    col += ubuf.coreColor.rgb * smoothstep(4.20, 9.00, v) * 0.38;

    fragColor = vec4(col, 1.0) * a * ubuf.qt_Opacity;
}
