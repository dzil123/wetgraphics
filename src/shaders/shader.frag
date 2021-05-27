#version 450

layout(location = 0) in vec2 uv;
layout(location = 0) out vec4 f_color;

layout(push_constant) uniform PushConstants {
    uint width;
    int pixels;
    uint front;  // bool
}
pushc;

layout(set = 0, binding = 0) uniform texture2D t_diffuse;
layout(set = 0, binding = 1) uniform sampler s_diffuse;

const float EPS = 0.000001;

#include <lygia/generative/noised.glsl>
#include <lygia/generative/random.glsl>
#include <lygia/generative/snoise.glsl>
#include <rand.glsl>

void main() {
    uint width2 = pushc.width * 2;  // uv is on pixel center
    float pixelsf = float(pushc.pixels * 2 - 1) / width2 + EPS;

    float f = 0.0;

    float v = uv.x;
    bool d;
    if (pushc.front != 0) {
        d = v <= (0.0 + pixelsf);
    } else {
        d = v >= (1.0 - pixelsf);
    }

    // f = random(uv);
    // f = snoise(uv * pushc.pixels) * .5 + .5;
    // f = noised(uv * pushc.pixels).x * .5 + .5;

    // float x = 0.0;
    // if (pushc.pixels > 400) {
    //     x = pushc.pixels;
    // } else if (pushc.pixels > 0) {
    //     x = 1.0;
    // }
    // f = randf(vec3(uv, x)).x;

    // uint x = 0;
    // if (pushc.pixels > 400) {
    //     // x = uint(uint_MAXf * float(pushc.pixels));
    //     // x = 0xffffffff;
    //     x = uint(uint_MAXf * 400u);
    // } else if (pushc.pixels > 300) {
    //     // x = 0xfffffffe;
    //     x = uint(uint_MAXf * 300u);
    // } else if (pushc.pixels > 200) {
    //     // x = 0xfffffffd;
    //     x = uint(uint_MAXf * 200u);
    // } else if (pushc.pixels > 100) {
    //     // x = 0xfffffffc;
    //     x = uint(uint_MAXf * 100u);
    // }

    // ok for some reason passing in pushc.pixels as a float causes anything >= 1 to be the same

    f = randf(vec3(uv, float(pushc.pixels) / pushc.width)).x;

    f_color = vec4(vec3(f), 1.0);
}
