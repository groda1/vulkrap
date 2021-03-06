#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(binding = 0) uniform UniformBufferObject {
    mat4 model;
    mat4 view;
    mat4 proj;
    float wobble;
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

    float wobble_x = sin(mvp.wobble + gl_VertexIndex) * 0.1;
    float wobble_y = cos(mvp.wobble + gl_VertexIndex) * 0.1;

    vec3 derp = vec3(inPosition.x + wobble_x, inPosition.y + wobble_y, inPosition.z);

    gl_Position = mvp.proj * mvp.view * mvp.model * vec4(derp, 1.0);
    fragColor = inColor;
    edgePosition = edge[gl_VertexIndex % 3];
}