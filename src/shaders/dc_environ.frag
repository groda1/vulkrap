#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;
layout(location = 2) in vec4 inColor;

layout(location = 0) out vec4 outColor;

const vec3 lightPos = vec3(-0.1, -0.1, -0.1);
const vec3 lightColor = vec3(1.0, 0.8, 0.6);
const float ambientFactor = 0.0;

void main() {
    float lightDistance = distance(lightPos, inPosition);
    vec3 lightVector = normalize(lightPos - inPosition);

    float diffuseFactor = max(dot(inNormal, lightVector), 0.0);
    diffuseFactor *= smoothstep(1.8, 1.0, lightDistance);
    vec3 diffuse = lightColor * diffuseFactor;

    vec3 eyeVector = -normalize(inPosition);
    vec3 reflectVector = normalize(reflect(-lightVector, inNormal));
    float specFactor = pow(max(dot(eyeVector, reflectVector), 0.0), 64);
    specFactor *= smoothstep(1.8, 1.0, lightDistance);
    vec3 specular = 1 * specFactor * lightColor;

    outColor = vec4(min(diffuse + ambientFactor, 1.0) * inColor.xyz + specular, 1.0);
}