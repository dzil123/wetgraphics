#version 450

layout(location=0) in vec3 uv;
layout(location=0) out vec4 f_color;

void main() {
//    f_color = vec4(0.3, 0.2, 0.1, 1.0);
    f_color = vec4(pow(uv, vec3(2.2)), 1.0);
}
