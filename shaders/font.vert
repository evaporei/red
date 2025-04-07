#version 330 core

void main() {
    float x = float(gl_VertexID & 1);
    float y = float((gl_VertexID >> 1) & 1);
    gl_Position = vec4(x - 0.5, y - 0.5, 0.0, 1.0);
}
