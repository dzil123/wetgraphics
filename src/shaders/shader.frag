#version 450

layout(location=0) in vec2 uv;
layout(location=0) out vec4 f_color;

layout( push_constant ) uniform _ {
    vec3 color;
} PushConstants;

void main() {
//    f_color = vec4(0.3, 0.2, 0.1, 1.0);
//    f_color = vec4(PushConstants.color, 1.0);
//    f_color = gl_FragCoord / vec4(vec2(1000.0), vec2(1.0));
    float x = uv.x / gl_FragCoord.x;
    float y = (1.0 - uv.y) / gl_FragCoord.y;
    float aspect = x / y;

//    f_color = vec4(vec3(aspect), 1.0);

    f_color = vec4(uv, 0.0, uv.x);
}
