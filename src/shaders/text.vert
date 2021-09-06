#version 450
#extension GL_ARB_separate_shader_objects : enable

layout (push_constant) uniform pushConstants {
    mat4 transform;
    vec3 color;
    int character;
} model;

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} vp;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec2 inTexCoord;

layout(location = 0) flat out vec3 fragColor;
layout(location = 1) out vec2 fragTexCoord;

const int WIDTH = 16;
const float CHAR_WIDTH = 1.0/16.0;
const float CHAR_HEIGHT = 1.0/6.0;

void main() {
    fragColor = model.color;

    int character = model.character - 32; // First ASCII character in the texture will be 32
    int offset_y = character / WIDTH;
    int offset_x = character % WIDTH;
    fragTexCoord = vec2(inTexCoord.x * CHAR_WIDTH + offset_x * CHAR_WIDTH, inTexCoord.y * CHAR_HEIGHT + offset_y * CHAR_HEIGHT);

    gl_Position = vp.proj * vp.view * model.transform * vec4(inPosition, 1.0);
}
