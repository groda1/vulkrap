#version 450
#extension GL_ARB_separate_shader_objects : enable

layout(binding = 0) uniform UniformBufferObject {
    mat4 view;
    mat4 proj;
} vp;

layout(location = 0) in vec3 inPosition;
layout(location = 1) in vec3 inNormal;

layout(location = 0) flat out vec3 fragColor;
layout(location = 1) out vec3 edgePosition;

vec3 edge[3] = vec3[](
vec3(1.0, 0.0, 0.0),
vec3(0.0, 1.0, 0.0),
vec3(0.0, 0.0, 1.0)
);

const vec3 light_dir = normalize(vec3(0.0, -1.0, -1.0));
const vec3 light_color = vec3(1.0, 1.0, 1.0);

const vec3 color = vec3(0.2, 0.5, 0.2);
const float ambient_strength = 0.0;

void main() {
    gl_Position = vp.proj * vp.view * vec4(inPosition, 1.0);
    edgePosition = edge[gl_VertexIndex % 3];

    float diffuse_factor = max(dot(inNormal, -light_dir), 0.0);
    vec3 diffuse = diffuse_factor * light_color;



    vec3 eyeVector = -normalize((vp.view * vec4(inPosition, 1.0)).xyz);
    vec3 reflectVectorWorld = normalize(reflect(light_dir, inNormal));
    vec3 reflectVector = vec3(vp.view * vec4(reflectVectorWorld, 0.0));

    float spec_factor = pow(max(dot(eyeVector, reflectVector), 0.0), 64);
    vec3 specular = 0.15 * spec_factor * light_color;


    fragColor = min(diffuse + ambient_strength, 1.0) * color + specular;
}