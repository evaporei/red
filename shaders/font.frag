#version 330 core

layout(location = 0) out vec4 outColor;

uniform sampler2D font;

in vec2 uv;

void main() {
    outColor = texture(font, uv);
}
