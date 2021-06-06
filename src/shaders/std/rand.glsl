/*

uvec4 rand (uvecN v);

 vec4 randf( vecN v);

*/

#ifndef STD_RAND
#define STD_RAND

#include <consts.glsl>

// http://www.jcgt.org/published/0009/03/02/
uvec4 pcg4d(uvec4 v) {
    v = v * 1664525u + 1013904223u;

    v.x += v.y * v.w;
    v.y += v.z * v.x;
    v.z += v.x * v.y;
    v.w += v.y * v.z;

    v ^= v >> 16u;

    v.x += v.y * v.w;
    v.y += v.z * v.x;
    v.z += v.x * v.y;
    v.w += v.y * v.z;

    return v;
}

uvec4 rand(uvec4 v) {
    return pcg4d(v);
}

uvec4 rand(uvec3 v) {
    return rand(uvec4(v, 0));
}

uvec4 rand(uvec2 v) {
    return rand(uvec4(v, 0, 0));
}

uvec4 rand(uint v) {
    return rand(uvec4(v, 0, 0, 0));
}

// randf best for 0.0 - 1.0  // todo fix

vec4 randf(vec4 v) {
    return rand(uvec4(v * uint_MAXf)) / uint_MAXf;
}

vec4 randf(vec3 v) {
    return randf(vec4(v, 0));
}

vec4 randf(vec2 v) {
    return randf(vec4(v, 0, 0));
}

vec4 randf(float v) {
    return randf(vec4(v, 0, 0, 0));
}

#endif
