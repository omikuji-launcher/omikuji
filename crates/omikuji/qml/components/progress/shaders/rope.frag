#version 440

layout(std140, binding = 0) uniform buf {
    mat4 qt_Matrix;
    float qt_Opacity;
    float lineWidth;
    vec2 resolution;
    vec4 lineColor;
    vec4 fillColor;
} ubuf;

layout(location = 0) in vec2 qt_TexCoord0;
layout(location = 0) out vec4 fragColor;

void main() {
    vec2 p = qt_TexCoord0 * ubuf.resolution;
    float lw = ubuf.lineWidth;
    float cy = ubuf.resolution.y * 0.5;
    float thick = cy - lw * 0.5;
    float tail = thick * 0.72;

    float r = mix(tail, thick, smoothstep(tail, tail + thick * 4.0, p.x));
    float d = p.x < tail ? length(p - vec2(tail, cy)) - tail : abs(p.y - cy) - r;

    float aa = fwidth(d) + 1e-4;
    float inside = 1.0 - smoothstep(-aa, aa, d);
    float edge = 1.0 - smoothstep(lw * 0.5 - aa, lw * 0.5 + aa, abs(d));

    float period = thick * 1.6;
    float s = (ubuf.resolution.x - p.x) + (p.y - cy) * 0.9;
    float ds = abs(fract(s / period) - 0.5) * period;
    float saa = fwidth(ds) + 1e-4;
    float twist = (1.0 - smoothstep(lw * 0.5 - saa, lw * 0.5 + saa, ds))
                * (1.0 - smoothstep(-lw, -lw * 0.5, d));

    float line = max(edge, twist * inside);
    vec3 rgb = mix(ubuf.fillColor.rgb, ubuf.lineColor.rgb, line);
    fragColor = vec4(rgb, 1.0) * max(inside, edge) * ubuf.qt_Opacity;
}
