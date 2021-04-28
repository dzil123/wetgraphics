#version 450

layout(location=0) in vec2 uv;
layout(location=0) out vec4 f_color;

layout( push_constant ) uniform _ {
    vec3 color;
} PushConstants;

void main() {
//    f_color = vec4(0.3, 0.2, 0.1, 1.0);
    f_color = vec4(PushConstants.color, 1.0);
}
