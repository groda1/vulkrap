#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} vp;

layout(location = 0) in vec2 inPosition;
layout(location = 1) in vec4 inColor;
layout(location = 2) in vec2 inTexCoord;

layout(location = 0) flat out vec4 fragColor;

void main() {
    fragColor = inColor;
    gl_Position = vp.proj * vp.view * vec4(inPosition, 0.0, 1.0);
}
