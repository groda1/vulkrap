#version 450
#extension GL_ARB_separate_shader_objects : enable

layout (push_constant) uniform pushConstants {
    mat4 transform;
    vec4 color;
} model;

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} vp;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;

layout(location = 0) flat out vec3 outColor;
layout(location = 1) out vec3 outPosition;
layout(location = 2) out vec3 outNormal;
layout(location = 3) out vec3 lightPosition;

void main() {
    mat4 mvp = vp.proj * vp.view * model.transform;

    gl_Position = mvp * vec4(inPosition, 1.0);
    outPosition = vec3(mvp * vec4(inPosition, 1.0));
    outNormal = vec3(mvp * vec4(inNormal, 0.0));
    lightPosition = vec3(vp.proj * vp.view * vec4(-1.0, 1.0, 0.0, 3.0));
    outColor = vec3(model.color);
}