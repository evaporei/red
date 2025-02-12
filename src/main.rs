use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::render::{Texture, WindowCanvas};
use sdl2::surface::Surface;
use stb_image::stb_image::stbi_load;
use std::ffi::CString;

const FONT_SCALE: f32 = 5.0;
const FONT_WIDTH: usize = 128;
const FONT_HEIGHT: usize = 64;

const FONT_COLS: usize = 18;
const FONT_ROWS: usize = 7;

const FONT_CHAR_WIDTH: usize = FONT_WIDTH / FONT_COLS;
const FONT_CHAR_HEIGHT: usize = FONT_HEIGHT / FONT_ROWS;

fn load_img(file_path: &str) -> (Vec<u8>, i32, i32) {
    let c_path = CString::new(file_path).unwrap();

    let mut width = 0;
    let mut height = 0;
    let mut channels = 0;
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

fn surface_from_file(pixels: &mut Vec<u8>, width: i32, height: i32) -> Result<Surface, String> {
    Surface::from_data(
        pixels,
        width as u32,
        height as u32,
        (4 * width) as u32,
        PixelFormatEnum::RGBA32,
    )
}

const ASCII_DISPLAY_LOW: u8 = 32;
const ASCII_DISPLAY_HIGH: u8 = 126;

struct Font<'a> {
    spritesheet: Texture<'a>,
    glyph_table: [Rect; (ASCII_DISPLAY_HIGH - ASCII_DISPLAY_LOW + 1) as usize],
}

impl<'a> Font<'a> {
    fn new(spritesheet: Texture<'a>) -> Self {
        let glyph_table = std::array::from_fn(|i| {
            let col = i % FONT_COLS;
            let row = i / FONT_COLS;

            Rect::new(
                col as i32 * FONT_CHAR_WIDTH as i32,
                row as i32 * FONT_CHAR_HEIGHT as i32,
                FONT_CHAR_WIDTH as u32,
                FONT_CHAR_HEIGHT as u32,
            )
        });

        Self {
            spritesheet,
            glyph_table,
        }
    }
}

#[derive(Copy, Clone)]
struct Vec2f {
    pub x: f32,
    pub y: f32,
}

fn vec2f(x: f32, y: f32) -> Vec2f {
    Vec2f { x, y }
}

fn render_char(
    canvas: &mut WindowCanvas,
    font: &Font,
    c: u8,
    pos: Vec2f,
    scale: f32,
) -> Result<(), String> {
    assert!(c >= ASCII_DISPLAY_LOW);
    assert!(c <= ASCII_DISPLAY_HIGH);
    let idx = (c - b' ') as usize;

    // view into the texture
    let src = font.glyph_table[idx];

    // where in the screen/window
    let dst = Rect::new(
        pos.x.floor() as i32,
        pos.y.floor() as i32,
        (FONT_CHAR_WIDTH as f32 * scale).floor() as u32,
        (FONT_CHAR_HEIGHT as f32 * scale).floor() as u32,
    );

    canvas.copy(&font.spritesheet, src, dst)
}

fn set_texture_color(texture: &mut Texture<'_>, color: Color) {
    texture.set_color_mod(color.r, color.g, color.b);
    texture.set_alpha_mod(color.a);
}

fn render_text(
    canvas: &mut WindowCanvas,
    font: &mut Font,
    text: &str,
    pos: Vec2f,
    color: Color,
    scale: f32,
) -> Result<(), String> {
    set_texture_color(&mut font.spritesheet, color);

    let mut pen = pos;
    for ch in text.bytes() {
        render_char(canvas, font, ch, pen, scale)?;
        pen.x += FONT_CHAR_WIDTH as f32 * scale;
    }
    Ok(())
}

fn render_cursor(canvas: &mut WindowCanvas, color: Color, cursor: usize) -> Result<(), String> {
    canvas.set_draw_color(color);
    canvas.fill_rect(Rect::new(
        (cursor as f32 * FONT_CHAR_WIDTH as f32 * FONT_SCALE).floor() as i32,
        0,
        (FONT_CHAR_WIDTH as f32 * FONT_SCALE) as u32,
        (FONT_CHAR_HEIGHT as f32 * FONT_SCALE) as u32,
    ))
}

fn main() -> Result<(), String> {
    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;

    let window = video_subsystem
        .window("red", 800, 600)
        .position_centered()
        .resizable()
        .build()
        .map_err(|e| e.to_string())?;

    let mut canvas = window
        .into_canvas()
        .accelerated()
        .build()
        .map_err(|e| e.to_string())?;

    let (mut pixels, width, height) = load_img("charmap-oldschool_white.png");
    let font_surface = surface_from_file(&mut pixels, width, height)?;
    let texture_creator = canvas.texture_creator();
    let font_texture = font_surface
        .as_texture(&texture_creator)
        .map_err(|e| e.to_string())?;

    let mut font = Font::new(font_texture);

    let mut event_pump = sdl_context.event_pump()?;

    let mut buffer = String::new();
    let mut cursor = 0;

    let mut quit = false;
    while !quit {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => quit = true,
                Event::KeyDown { keycode, .. } => match keycode {
                    Some(key) => match key {
                        Keycode::Backspace => {
                            if buffer.pop().is_some() {
                                cursor -= 1;
                            }
                        }
                        Keycode::Left => {
                            if cursor > 0 {
                                cursor -= 1;
                            }
                        }
                        Keycode::Right => {
                            if cursor < buffer.len() {
                                cursor += 1;
                            }
                        }
                        _ => {}
                    },
                    _ => {}
                },
                Event::TextInput { text, .. } => {
                    buffer.push_str(&text);
                    cursor += text.len();
                }
                _ => {}
            }
        }

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();

        render_text(
            &mut canvas,
            &mut font,
            &buffer,
            vec2f(0.0, 0.0),
            Color::WHITE,
            FONT_SCALE,
        )?;
        render_cursor(&mut canvas, Color::WHITE, cursor)?;

        canvas.present();
    }

    Ok(())
}
