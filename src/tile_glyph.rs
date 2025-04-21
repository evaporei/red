use std::{
    ffi::{c_void, CString},
    mem::offset_of,
};

use gl::types::GLuint;
use stb_image::stb_image::stbi_load;

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

fn load_img(file_path: &str) -> (Vec<u8>, i32, i32) {
    let c_path = CString::new(file_path).unwrap();

    let mut width = 0;
    let mut height = 0;
    let mut channels = 3;
    let stbi_rgb_alpha = 4;

    let pixels = unsafe {
        stbi_load(
            c_path.as_ptr(),
            &mut width,
            &mut height,
            &mut channels,
            stbi_rgb_alpha,
        )
    };

    if pixels.is_null() {
        panic!("image is null after load");
    }

    (
        unsafe { std::slice::from_raw_parts(pixels, (width * height * 4) as usize) }.to_vec(),
        width,
        height,
    )
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
    pub fn load_texture_atlas(&self, file_path: &str) {
        let mut font_texture = 0;
        let (mut pixels, width, height) = load_img(file_path);
        unsafe {
            gl::ActiveTexture(gl::TEXTURE0);
            gl::GenTextures(1, &mut font_texture);
            gl::BindTexture(gl::TEXTURE_2D, font_texture);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::NEAREST as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::NEAREST as i32);

            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::CLAMP_TO_EDGE as i32);
            gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::CLAMP_TO_EDGE as i32);

            gl::TexImage2D(
                gl::TEXTURE_2D,
                0,
                gl::RGBA as i32,
                width,
                height,
                0,
                gl::RGBA,
                gl::UNSIGNED_BYTE,
                pixels.as_mut_ptr() as *mut c_void,
            );
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
