#version 450

layout(location=0) in vec2 uv;
layout(location=0) out vec4 f_color;

layout( push_constant ) uniform PushConstants {
    uint width;
    int pixels;
    uint front; // bool
} pushc;

layout(set = 0, binding = 0) uniform texture2D t_diffuse;
layout(set = 0, binding = 1) uniform sampler s_diffuse;

const float EPS = 0.000001;

void main() {
    uint width2 = pushc.width * 2; // uv is on pixel center
    float pixelsf = float(pushc.pixels * 2 - 1) / width2 + EPS;

    float f = 0.0;

    float v = uv.x;
    bool d;
    if (pushc.front != 0) {
        d = v <= (0.0 + pixelsf);
    } else {
        d = v >= (1.0 - pixelsf);
    }

    // if (d) {
    //     discard;
    // }

    // f = float(1.0);

    f = float(d);

    f_color = vec4(vec3(f), 1.0);
}
