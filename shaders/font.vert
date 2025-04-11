#version 330 core

out vec2 uv;

void main() {
    float x = float(gl_VertexID & 1);
    float y = float((gl_VertexID >> 1) & 1);
    uv = vec2(x, y);
    gl_Position = vec4(x - 0.5, y - 0.5, 0.0, 1.0);
}
