#version 450

#extension GL_EXT_samplerless_texture_functions : require

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 f_color;

layout(set = 0, binding = 0) uniform texture2D input_tex;
layout(set = 0, binding = 1) uniform sampler input_smp;

layout(std430, push_constant) uniform PushConstants {
    vec3 foreground;
    vec3 background;
    bool front;
    float offset;
}
pushc;

// todo: add to stdlib
float saturate(float x) {
    return clamp(x, 0.0, 1.0);
}

#include <consts.glsl>
#include <rand.glsl>

void main() {
    // uvec4 pixel = texelFetch(input_tex, ivec2(uv * vec2(pushc.size)), 0);

    float f = texture(sampler2D(input_tex, input_smp), uv).x;
    // f = saturate(f);

    f += pushc.offset;
    if (f > 1.0 + EPSILON) {
        f -= 1.0;
    }

    if (pushc.front) {
        f = 1.0 - f;
    }

    vec3 color = mix(pushc.background, pushc.foreground, f);

    f_color = vec4(color, 1.0);
}
