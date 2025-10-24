#version 460

layout (location = 0) out vec2 texPosition;

void main() {

    // Fullscreen triangle, covering full [0,1] uv range.
    float triangle_len = 3.0;
    const vec2 positions[3] = vec2[](
        vec2(-1.0, -1.0),
        vec2(-1.0, triangle_len),
        vec2(triangle_len, -1.0)
    );

    const vec2 uv[3] = vec2[](
        vec2(0.0, 0.0),
        vec2(0.0, 2.0),
        vec2(2.0, 0.0)
    );

    texPosition = uv[gl_VertexIndex];
    gl_Position = vec4(positions[gl_VertexIndex], 0.0, 1.0);
}