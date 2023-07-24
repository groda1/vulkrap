#version 450
#extension GL_ARB_separate_shader_objects : enable

layout (push_constant) uniform pushConstants {
    vec2 position;
    vec2 size;
    vec4 color;
} model;

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} vp;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec2 inTexCoord;

layout(location = 0) flat out vec4 fragColor;
layout(location = 1) out vec2 fragTexCoord;

void main() {
    fragColor = model.color;
    fragTexCoord = inTexCoord;

    vec4 position = vec4((inPosition.x * model.size.x) + model.position.x, (inPosition.y * model.size.y ) + model.position.y, 0.0, 1.0);

    gl_Position = vp.proj * vp.view * position;
}
