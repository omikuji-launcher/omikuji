#version 440

layout(std140, binding = 0) uniform buf {
    mat4 qt_Matrix;
    float qt_Opacity;
    float time;
    vec2 resolution;
    vec4 gateColor;
    vec4 gateCoreColor;
    vec4 trimColor;
    vec4 spiritColor;
    vec4 spiritCoreColor;
} ubuf;

layout(location = 0) in vec2 qt_TexCoord0;
layout(location = 0) out vec4 fragColor;

float sdBox(vec2 p, vec2 b, float r) {
    vec2 d = abs(p) - b + r;
    return length(max(d, 0.0)) + min(max(d.x, d.y), 0.0) - r;
}

float sdSegment(vec2 p, vec2 a, vec2 b, float r) {
    vec2 pa = p - a;
    vec2 ba = b - a;
    float h = clamp(dot(pa, ba) / dot(ba, ba), 0.0, 1.0);
    return length(pa - ba * h) - r;
}

float smin(float a, float b, float k) {
    float h = clamp(0.5 + 0.5 * (b - a) / k, 0.0, 1.0);
    return mix(b, a, h) - k * h * (1.0 - h);
}

float torii(vec2 p) {
    float d = sdSegment(p, vec2(-0.335, -0.440), vec2(-0.300, 0.345), 0.042);
    d = min(d, sdSegment(p, vec2(0.335, -0.440), vec2(0.300, 0.345), 0.042));
    d = smin(d, sdBox(p - vec2(0.0, 0.165), vec2(0.360, 0.034), 0.014), 0.022);
    d = smin(d, sdBox(p - vec2(0.0, 0.258), vec2(0.030, 0.092), 0.012), 0.022);

    vec2 s = p - vec2(0.0, 0.318);
    s.y -= p.x * p.x * 0.10;
    d = smin(d, sdBox(s, vec2(0.435, 0.026), 0.012), 0.022);

    vec2 k = p - vec2(0.0, 0.396);
    k.y -= p.x * p.x * 0.24;
    return smin(d, sdBox(k, vec2(0.500, 0.038), 0.018), 0.026);
}

float toriiTrim(vec2 p) {
    vec2 k = p - vec2(0.0, 0.396);
    k.y -= p.x * p.x * 0.24;
    float d = sdBox(k, vec2(0.500, 0.038), 0.018);
    vec2 m = vec2(abs(p.x), p.y);
    d = min(d, sdBox(m - vec2(0.335, 0.165), vec2(0.032, 0.046), 0.010));
    return min(d, sdBox(m - vec2(0.331, -0.352), vec2(0.058, 0.034), 0.012));
}

float obake(vec2 p, float t, float wave) {
    float d = min(length(p) - 0.082, sdBox(p - vec2(0.0, -0.048), vec2(0.082, 0.048), 0.0));
    d = max(d, -0.098 + sin(p.x * 30.0 - t * 3.2) * 0.015 - p.y);

    vec2 shoulder = vec2(0.060, -0.016);
    return smin(d, sdSegment(p, shoulder, shoulder + vec2(cos(wave), sin(wave)) * 0.088, 0.021), 0.016);
}

void main() {
    float aspect = ubuf.resolution.x / max(ubuf.resolution.y, 1.0);
    vec2 uv = vec2((qt_TexCoord0.x - 0.5) * aspect, 0.5 - qt_TexCoord0.y);

    float t = ubuf.time;

    // 0.90 or the kasagi ends clip
    vec2 p = uv / 0.90;

    float d = torii(p);
    float gateAA = max(fwidth(d), 1e-4);
    float gateA = 1.0 - smoothstep(-gateAA, gateAA, d);

    float wave = 1.05 + sin(t * 6.20) * 0.40;

    vec2 q = uv - vec2(sin(t * 0.90) * 0.013, -0.118 + sin(t * 1.55) * 0.018);
    float lean = sin(t * 6.20) * 0.055;
    q = mat2(cos(lean), -sin(lean), sin(lean), cos(lean)) * q / 1.18;

    float ds = obake(q, t, wave);
    float spiritAA = max(fwidth(ds), 1e-4);
    float spiritA = 1.0 - smoothstep(-spiritAA, spiritAA, ds);

    vec3 spiritCol = mix(ubuf.spiritColor.rgb, ubuf.spiritCoreColor.rgb, smoothstep(0.0, -0.055, ds));

    float squish = mix(1.0, 9.0, step(0.965, sin(t * 3.10)));
    vec2 eye = abs(q - vec2(0.0, 0.016)) - vec2(0.033, 0.0);
    eye.y *= squish;
    spiritCol = mix(spiritCol, vec3(0.15, 0.12, 0.16),
                    1.0 - smoothstep(0.011, 0.023, length(eye)));

    vec2 blush = abs(q - vec2(0.0, -0.020)) - vec2(0.056, 0.0);
    spiritCol = mix(spiritCol, vec3(0.90, 0.35, 0.45),
                    (1.0 - smoothstep(0.0, 0.028, length(blush))) * 0.45);

    float lit = exp(-length(uv - vec2(0.0, -0.118)) * 4.2);
    vec3 gateCol = mix(ubuf.gateColor.rgb, ubuf.gateCoreColor.rgb,
                       smoothstep(0.055, 0.0, -d) * 0.55 + lit * 0.35);

    // the rim keeps trim visible on a dark card
    // own fwidth, the warp skews this field
    float trimD = toriiTrim(p) + 0.014;
    float trimAA = max(fwidth(trimD), 1e-4);
    gateCol = mix(gateCol, ubuf.trimColor.rgb, 1.0 - smoothstep(-trimAA, trimAA, trimD));

    float behind = 1.0 - gateA;
    vec3 col = spiritCol * spiritA * behind;
    float a = spiritA * behind;

    col = col * (1.0 - gateA) + gateCol * gateA;
    a = a * (1.0 - gateA) + gateA;

    fragColor = vec4(col, a) * ubuf.qt_Opacity;
}
