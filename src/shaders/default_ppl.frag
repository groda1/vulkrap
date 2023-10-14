#version 450
#extension GL_ARB_separate_shader_objects : enable


layout(location = 0) flat in vec3 inColor;
layout(location = 1) in vec3 inPosition;
layout(location = 2) in vec3 inNormal;
layout(location = 3) in vec3 inLightPosition;

layout(location = 0) out vec4 outColor;


const float ambient_strength = 0.0;
const vec3 lightColor = vec3(1.0, 0.7, 0.7);

void main() {
    vec3 lightVector = normalize(inLightPosition - inPosition);
    float diffuse_factor = max(dot(normalize(inNormal), lightVector), 0.0);
    vec3 diffuse = diffuse_factor * lightColor;

    vec3 eyeVector = -normalize(inPosition);
    vec3 reflectVector = normalize(reflect(-lightVector, inNormal));
    float spec_factor = pow(max(dot(eyeVector, reflectVector), 0.0), 64);
    vec3 specular = 1 * spec_factor * lightColor;

    outColor = vec4(min(diffuse + ambient_strength, 1.0) * inColor + specular, 1.0);

}