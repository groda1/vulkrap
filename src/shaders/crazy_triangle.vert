#version 450
#extension GL_ARB_separate_shader_objects : enable

layout (push_constant) uniform pushConstants {
    mat4 transform;
    float wobble;
} model;

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} mvp;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inColor;

layout(location = 0) out vec3 fragColor;
layout(location = 1) out vec3 edgePosition;

vec3 edge[3] = vec3[](
vec3(1.0, 0.0, 0.0),
vec3(0.0, 1.0, 0.0),
vec3(0.0, 0.0, 1.0)
);

void main() {

    float wobble_x = cos(model.wobble + gl_VertexIndex) * 0.1;
    float wobble_y = sin(model.wobble + gl_VertexIndex) * 0.1;

    vec3 wobbled_position = vec3(inPosition.x + wobble_x, inPosition.y + wobble_y, inPosition.z);

    gl_Position = mvp.proj * mvp.view * model.transform * vec4(wobbled_position, 1.0);
    fragColor = inColor;
    edgePosition = edge[gl_VertexIndex % 3];
}