#version 440

layout(std140, binding = 0) uniform buf {
    mat4 qt_Matrix;
    float qt_Opacity;
    float offset;
    float period;
    vec2 resolution;
    vec4 fillColor;
    vec4 stripeColor;
} ubuf;

layout(location = 0) in vec2 qt_TexCoord0;
layout(location = 0) out vec4 fragColor;

void main() {
    vec2 p = qt_TexCoord0 * ubuf.resolution;
    float r = ubuf.resolution.y * 0.5;
    vec2 axis = vec2(clamp(p.x, r, max(r, ubuf.resolution.x - r)), r);
    float d = length(p - axis) - r;
    float aa = fwidth(d) + 1e-4;
    float mask = 1.0 - smoothstep(-aa, aa, d);

    float ds = abs(fract((p.x - p.y - ubuf.offset) / ubuf.period) - 0.5) * ubuf.period;
    float saa = fwidth(ds) + 1e-4;
    float band = 1.0 - smoothstep(ubuf.period * 0.25 - saa, ubuf.period * 0.25 + saa, ds);

    vec3 rgb = mix(ubuf.fillColor.rgb, ubuf.stripeColor.rgb, band);
    fragColor = vec4(rgb, 1.0) * mask * ubuf.qt_Opacity;
}
