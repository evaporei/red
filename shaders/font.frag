#version 330 core

in vec2 uv;

uniform sampler2D font;
uniform float time;

layout(location = 0) out vec4 outColor;

void main() {
    outColor = texture(font, uv) *
            vec4((sin(uv.x + time) + 1.0) / 2.0,
                (cos(uv.y + time) + 1.0) / 2.0,
                (sin(uv.x + time) + 1.0) / 2.0, 1.0);
}
