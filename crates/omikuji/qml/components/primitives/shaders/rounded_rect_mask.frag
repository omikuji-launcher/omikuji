#version 440

layout(location = 0) in vec2 qt_TexCoord0;
layout(location = 0) out vec4 fragColor;

layout(std140, binding = 0) uniform buf {
    mat4 qt_Matrix;
    float qt_Opacity;
    vec2 itemSize;
    vec4 maskRect;
    float radius;
} ubuf;

layout(binding = 1) uniform sampler2D source;

float roundedRectDistance(vec2 p, vec2 halfSize, float r) {
    vec2 q = abs(p) - halfSize + r;
    return length(max(q, 0.0)) + min(max(q.x, q.y), 0.0) - r;
}

void main() {
    vec2 halfSize = ubuf.maskRect.zw * 0.5;
    vec2 p = qt_TexCoord0 * ubuf.itemSize - ubuf.maskRect.xy - halfSize;
    float d = roundedRectDistance(p, halfSize, min(ubuf.radius, min(halfSize.x, halfSize.y)));
    float coverage = clamp(0.5 - d / max(fwidth(d), 1e-4), 0.0, 1.0);
    fragColor = texture(source, qt_TexCoord0) * coverage * ubuf.qt_Opacity;
}
