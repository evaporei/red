use std::{ffi::c_void, mem::offset_of};

use crate::{editor::Editor, v2, vector::Vector2, Color, BLACK, WHITE};

#[repr(C)]
pub struct TileGlyph {
    tile: Vector2<i32>,
    ch: i32,
    fg_color: Color,
    bg_color: Color,
}

pub struct GlAttrib {
    pub r#type: gl::types::GLenum,
    pub comps: i32,
    pub normalized: gl::types::GLboolean,
    pub stride: i32,
    pub offset: usize,
}

impl TileGlyph {
    pub const fn gl_attributes() -> [GlAttrib; 4] {
        let stride = size_of::<TileGlyph>() as i32;
        let normalized = gl::FALSE;
        [
            GlAttrib {
                r#type: gl::INT,
                comps: 2,
                normalized,
                stride,
                offset: offset_of!(TileGlyph, tile),
            },
            GlAttrib {
                r#type: gl::INT,
                comps: 1,
                normalized,
                stride,
                offset: offset_of!(TileGlyph, ch),
            },
            GlAttrib {
                r#type: gl::FLOAT,
                comps: 4,
                normalized,
                stride,
                offset: offset_of!(TileGlyph, fg_color),
            },
            GlAttrib {
                r#type: gl::FLOAT,
                comps: 4,
                normalized,
                stride,
                offset: offset_of!(TileGlyph, bg_color),
            },
        ]
    }
}

pub const TILE_GLYPH_BUFF_CAP: usize = 640 * 1024;
type TileGlyphBuffer = Vec<TileGlyph>;

pub fn render_line(
    tile_glyph_buf: &mut TileGlyphBuffer,
    line: &str,
    tile: Vector2<i32>,
    fg_color: Color,
    bg_color: Color,
) {
    for (i, ch) in line.chars().enumerate() {
        let tile_glyph = TileGlyph {
            tile: tile + v2!(i as i32, 0),
            ch: ch as i32,
            fg_color,
            bg_color,
        };
        tile_glyph_buf.push(tile_glyph);
    }
}

pub fn gl_render_cursor(tile_glyph_buf: &mut TileGlyphBuffer, editor: &Editor) {
    render_line(
        tile_glyph_buf,
        &editor.char_at_cursor().unwrap_or(' ').to_string(),
        v2!(editor.cursor.x as i32, -(editor.cursor.y as i32)),
        BLACK,
        WHITE,
    );
}

pub fn sync(tile_glyph_buf: &TileGlyphBuffer) {
    unsafe {
        gl::BufferSubData(
            gl::ARRAY_BUFFER,
            0,
            (tile_glyph_buf.len() * size_of::<TileGlyph>()) as isize,
            tile_glyph_buf.as_ptr() as *const c_void,
        );
    }
}
