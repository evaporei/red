#version 330 core

#define FONT_WIDTH 128
#define FONT_HEIGHT 64

#define FONT_COLS 18
#define FONT_ROWS 7

#define FONT_CHAR_WIDTH  (FONT_WIDTH  / FONT_COLS)
#define FONT_CHAR_HEIGHT (FONT_HEIGHT / FONT_ROWS)

#define FONT_CHAR_WIDTH_UV  (float(FONT_CHAR_WIDTH) / float(FONT_WIDTH))
#define FONT_CHAR_HEIGHT_UV (float(FONT_CHAR_HEIGHT) / float(FONT_HEIGHT))

#define ASCII_DISPLAY_LOW 32
#define ASCII_DISPLAY_HIGH 126

uniform sampler2D font;
uniform float time;

in vec2 uv;
in float glyph_ch;
in vec4 glyph_color;

layout(location = 0) out vec4 outColor;

void main() {
    // outColor = vec4(1.0, 0.0, 0.0, 1.0);

    // outColor = vec4(uv.x, uv.y, 0.0, 1.0);

    int ch = int(glyph_ch);
    if (!(ASCII_DISPLAY_LOW <= ch && ch <= ASCII_DISPLAY_HIGH)) {
        ch = 63; // '?'
    }

    int index = ch - 32; // ' ' (space)
    float x = float(index % FONT_COLS) * FONT_CHAR_WIDTH_UV;
    float y = float(index / FONT_COLS) * FONT_CHAR_HEIGHT_UV;
    vec2 pos = vec2(x, y + FONT_CHAR_HEIGHT_UV);
    vec2 size = vec2(FONT_CHAR_WIDTH_UV, -FONT_CHAR_HEIGHT_UV);
    vec2 t = pos + size * uv;
    // vec4 tex_color = texture(font, t);
    // outColor = tex_color * glyph_color;
    outColor = texture(font, t) * glyph_color;

    // vec4 cool_color = vec4((sin(uv.x + time) + 1.0) / 2.0,
    //                 (cos(uv.y + time) + 1.0) / 2.0,
    //                 (sin(uv.x + time) + 1.0) / 2.0, 1.0);
    // outColor = texture(font, t) * cool_color;
}
