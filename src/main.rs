use sdl2::event::Event;
use sdl2::pixels::Color;
use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::surface::Surface;
use stb_image::stb_image::stbi_load;
use std::ffi::CString;

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
    spritesheet: sdl2::render::Texture<'a>,
    glyph_table: [Rect; (ASCII_DISPLAY_HIGH - ASCII_DISPLAY_LOW + 1) as usize],
}

impl<'a> Font<'a> {
    fn new(spritesheet: sdl2::render::Texture<'a>) -> Self {
        let glyph_table = std::array::from_fn(|i| {
            let col = i % FONT_COLS;
            let row = i / FONT_COLS;

            // view into the texture
            let src = Rect::new(
                col as i32 * FONT_CHAR_WIDTH as i32,
                row as i32 * FONT_CHAR_HEIGHT as i32,
                FONT_CHAR_WIDTH as u32,
                FONT_CHAR_HEIGHT as u32,
            );
            src
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
    canvas: &mut sdl2::render::WindowCanvas,
    font: &Font,
    c: u8,
    pos: Vec2f,
    scale: f32,
) -> Result<(), String> {
    assert!(c >= ASCII_DISPLAY_LOW);
    assert!(c <= ASCII_DISPLAY_HIGH);
    let idx = (c - b' ') as usize;

    // where in the screen/window
    let dst = Rect::new(
        pos.x.floor() as i32,
        pos.y.floor() as i32,
        (FONT_CHAR_WIDTH as f32 * scale).floor() as u32,
        (FONT_CHAR_HEIGHT as f32 * scale).floor() as u32,
    );

    canvas.copy(&font.spritesheet, font.glyph_table[idx], dst)
}

fn render_text(
    canvas: &mut sdl2::render::WindowCanvas,
    font: &mut Font,
    text: &str,
    pos: Vec2f,
    color: u32,
    scale: f32,
) -> Result<(), String> {
    font.spritesheet.set_color_mod(
        ((color >> (8 * 3)) & 0xff) as u8,
        ((color >> (8 * 2)) & 0xff) as u8,
        ((color >> (8 * 1)) & 0xff) as u8,
    );
    font.spritesheet.set_alpha_mod(((color >> (8 * 0)) & 0xff) as u8);

    let mut pen = pos;
    for ch in text.bytes() {
        render_char(canvas, font, ch, pen, scale)?;
        pen.x += FONT_CHAR_WIDTH as f32 * scale;
    }
    Ok(())
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

    let mut quit = false;
    while !quit {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => quit = true,
                _ => {}
            }
        }

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();

        render_text(
            &mut canvas,
            &mut font,
            "hello world!",
            vec2f(0.0, 0.0),
            0xff0000ff,
            5.0,
        )?;

        canvas.present();
    }

    Ok(())
}
