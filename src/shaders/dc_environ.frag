#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec4 inColor;

layout(location = 0) out vec4 outColor;

const vec3 lightPos = vec3(0.0, -0.1, 0.0);
const vec3 lightColor = vec3(1.0, 0.8, 0.6);
const float ambientFactor = 0.02;

void main() {

    float lightDistance = distance(lightPos, inPosition);
    vec3 lightVec = normalize(lightPos - inPosition);

    float diffuseFactor = 0.0;
        diffuseFactor = max(dot(inNormal, lightVec), 0.0);
    diffuseFactor *= smoothstep(1.8, 1.5, lightDistance);

    vec3 diffuse = lightColor * diffuseFactor;
    vec3 color = min(diffuse + ambientFactor, 1.0) * inColor.xyz;
    outColor = vec4(color, inColor.a);
}