#version 460

layout (binding = 0) uniform sampler2D depthTexture;

layout (location = 0) in vec2 texPosition;

layout (location = 0) out vec4 outColor;

void main() {
    float shade = texture(depthTexture, texPosition).r;
    outColor = vec4(vec3(shade), 1.0);
}