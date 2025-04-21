use std::{ffi::c_void, mem::offset_of};

use gl::types::GLuint;

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

const TILE_GLYPH_BUFF_CAP: usize = 640 * 1024;

pub struct TileGlyphBuffer(Vec<TileGlyph>);

impl TileGlyphBuffer {
    pub fn new() -> Self {
        Self(Vec::with_capacity(TILE_GLYPH_BUFF_CAP))
    }
    pub fn gl_init(&self) {
        unsafe {
            let mut vao: GLuint = 0;
            gl::GenVertexArrays(1, &mut vao);
            gl::BindVertexArray(vao);
        }

        unsafe {
            let mut vbo: GLuint = 0;
            gl::GenBuffers(1, &mut vbo);
            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
            gl::BufferData(
                gl::ARRAY_BUFFER,
                size_of::<[TileGlyph; TILE_GLYPH_BUFF_CAP]>() as isize,
                self.as_ptr() as *const std::ffi::c_void,
                gl::DYNAMIC_DRAW,
            );
        }

        for (i, attrib) in TileGlyph::gl_attributes().into_iter().enumerate() {
            let index = i as u32;
            let offset = attrib.offset as *const usize as *const std::ffi::c_void;
            unsafe {
                gl::EnableVertexAttribArray(index);
                match attrib.r#type {
                    gl::FLOAT => {
                        gl::VertexAttribPointer(
                            index,
                            attrib.comps,
                            attrib.r#type,
                            attrib.normalized,
                            attrib.stride,
                            offset,
                        );
                    }
                    gl::INT => {
                        gl::VertexAttribIPointer(
                            index,
                            attrib.comps,
                            attrib.r#type,
                            attrib.stride,
                            offset,
                        );
                    }
                    _ => unimplemented!("handle new gl attribute type"),
                }
                gl::VertexAttribDivisor(index, 1);
            }
        }
    }
    pub fn render_line(
        &mut self,
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
            self.push(tile_glyph);
        }
    }

    pub fn gl_render_cursor(&mut self, editor: &Editor) {
        self.render_line(
            &editor.char_at_cursor().unwrap_or(' ').to_string(),
            v2!(editor.cursor.x as i32, -(editor.cursor.y as i32)),
            BLACK,
            WHITE,
        );
    }

    pub fn sync(&self) {
        unsafe {
            gl::BufferSubData(
                gl::ARRAY_BUFFER,
                0,
                (self.len() * size_of::<TileGlyph>()) as isize,
                self.as_ptr() as *const c_void,
            );
        }
    }
}

use std::ops::Deref;

impl Deref for TileGlyphBuffer {
    type Target = Vec<TileGlyph>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

use std::ops::DerefMut;

impl DerefMut for TileGlyphBuffer {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
