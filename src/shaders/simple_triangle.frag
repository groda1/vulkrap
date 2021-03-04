#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 fragColor;

layout(location = 0) out vec4 outColor;

void main() {
    if (fragColor.r + fragColor.b > 0.98) {
        outColor = vec4(1.0, 1.0, 1.0, 1.0);
    } else if (fragColor.g + fragColor.b > 0.98) {
        outColor = vec4(1.0, 1.0, 1.0, 1.0);
    } else if (fragColor.g + fragColor.r > 0.98) {
        outColor = vec4(1.0, 1.0, 1.0, 1.0);
    } else {
        outColor = vec4(fragColor, 1.0);
    }
}