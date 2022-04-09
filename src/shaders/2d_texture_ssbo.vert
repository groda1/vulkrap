#version 450
#extension GL_ARB_separate_shader_objects : enable

struct instance_data {
    vec2 position;
    vec2 size;
    vec4 color;
};

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} vp;

layout(std430, binding = 2) buffer StorageBufferObject {
    instance_data instances[];
} text_data;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec2 inTexCoord;

layout(location = 0) flat out vec4 fragColor;
layout(location = 1) out vec2 fragTexCoord;

void main() {
    fragColor = text_data.instances[gl_InstanceIndex].color;
    fragTexCoord = inTexCoord;

    vec2 quad_position = text_data.instances[gl_InstanceIndex].position;
    vec2 size = text_data.instances[gl_InstanceIndex].size;

    vec4 position = vec4((inPosition.x * size.x) + quad_position.x, (inPosition.y *size.y ) + quad_position.y, 0.0, 1.0);

    gl_Position = vp.proj * vp.view * position;
}
