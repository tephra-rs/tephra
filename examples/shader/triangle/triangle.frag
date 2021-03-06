#version 430
#extension GL_ARB_separate_shader_objects : enable
#extension GL_ARB_shading_language_420pack : enable

layout (location = 0) in vec4 o_color;
layout (location = 0) out vec4 uFragColor;

layout(binding = 0 , set = 0) buffer readonly Color {
    vec4 color;
};

void main() {
    uFragColor = o_color;
}
