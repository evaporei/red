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

#[derive(Default, Copy, Clone)]
struct Vector2<T> {
    pub x: T,
    pub y: T,
}

impl<T> Vector2<T> {
    fn new(x: T, y: T) -> Vector2<T> {
        Vector2 { x, y }
    }
}

fn render_char(
    canvas: &mut WindowCanvas,
    font: &Font,
    c: u8,
    pos: Vector2<f32>,
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
    editor: &Editor,
    color: Color,
    scale: f32,
) -> Result<(), String> {
    set_texture_color(&mut font.spritesheet, color);

    let mut pen = Vector2::new(0.0, 0.0);
    for line in &editor.lines {
        for ch in line.chars.bytes() {
            render_char(canvas, font, ch, pen, scale)?;
            pen.x += FONT_CHAR_WIDTH as f32 * scale;
        }
        pen.x = 0.0;
        pen.y += FONT_CHAR_HEIGHT as f32 * scale;
    }
    Ok(())
}

fn render_cursor(
    canvas: &mut WindowCanvas,
    font: &mut Font,
    editor: &Editor,
) -> Result<(), String> {
    let pos = Vector2::new(
        editor.cursor.x as f32 * FONT_CHAR_WIDTH as f32 * FONT_SCALE,
        editor.cursor.y as f32 * FONT_CHAR_HEIGHT as f32 * FONT_SCALE,
    );

    canvas.set_draw_color(Color::WHITE);
    canvas.fill_rect(Rect::new(
        (pos.x).floor() as i32,
        pos.y.floor() as i32,
        (FONT_CHAR_WIDTH as f32 * FONT_SCALE) as u32,
        (FONT_CHAR_HEIGHT as f32 * FONT_SCALE) as u32,
    ))?;

    set_texture_color(&mut font.spritesheet, Color::RGB(0, 0, 0));
    if editor.cursor.x < editor.lines[editor.cursor.y].chars.len() {
        render_char(
            canvas,
            font,
            editor.lines[editor.cursor.y]
                .chars
                .bytes()
                .nth(editor.cursor.x)
                .unwrap(),
            pos,
            FONT_SCALE,
        )?;
    }

    Ok(())
}

#[derive(Default)]
struct Line {
    chars: String,
}

impl Line {
    fn insert(&mut self, text: &str, col: usize) {
        self.chars.insert_str(col, text);
    }
    fn remove(&mut self, col: usize) {
        if !self.chars.is_empty() {
            self.chars.remove(col);
        }
    }
}

#[derive(Default)]
struct Editor {
    filepath: Option<PathBuf>,
    lines: Vec<Line>,
    cursor: Vector2<usize>,
}

use std::fs::File;
use std::io;

fn read_lines<P>(filename: P) -> io::Result<io::Lines<io::BufReader<File>>>
where
    P: AsRef<Path>,
{
    let file = File::open(filename)?;
    Ok(io::BufReader::new(file).lines())
}

impl Editor {
    fn new() -> Self {
        Self {
            filepath: None,
            lines: vec![Line::default()],
            cursor: Vector2::new(0, 0),
        }
    }
    fn from_filepath(filepath: String) -> std::io::Result<Self> {
        let filepath = PathBuf::from(filepath);
        let mut editor = Self::default();
        for line in read_lines(filepath)? {
            let mut chars = line?;
            if chars.ends_with('\n') {
                chars.pop();
            }
            editor.lines.push(Line { chars });
        }
        Ok(editor)
    }
    fn save(&self) -> std::io::Result<()> {
        let mut file = std::fs::File::options().write(true).truncate(true).open(
            self.filepath
                .as_ref()
                .unwrap_or(&PathBuf::from_str("output").unwrap()),
        )?;
        for line in &self.lines {
            file.write_all(&line.chars.as_bytes())?;
            file.write(&[b'\n'])?;
        }
        Ok(())
    }
}

use std::io::BufRead;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;

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
    let mut font_surface = surface_from_file(&mut pixels, width, height)?;
    font_surface.set_color_key(true, Color::RGBA(0, 0, 0, 0))?;
    let texture_creator = canvas.texture_creator();
    let font_texture = font_surface
        .as_texture(&texture_creator)
        .map_err(|e| e.to_string())?;

    let mut font = Font::new(font_texture);

    let mut event_pump = sdl_context.event_pump()?;

    let mut editor = if let Some(filepath) = std::env::args().skip(1).next() {
        Editor::from_filepath(filepath).map_err(|e| e.to_string())?
    } else {
        Editor::new()
    };

    let mut quit = false;
    while !quit {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } => quit = true,
                Event::KeyDown { keycode, .. } => match keycode {
                    Some(key) => match key {
                        Keycode::F2 => match editor.save() {
                            Ok(_) => println!("saved file!"),
                            Err(err) => eprintln!("{}", err),
                        },
                        Keycode::Backspace => {
                            if editor.cursor.x == 0 && editor.cursor.y > 0 {
                                let right_side = editor.lines.remove(editor.cursor.y);
                                editor.cursor.y -= 1;
                                editor.cursor.x = editor.lines[editor.cursor.y].chars.len();
                                editor.lines[editor.cursor.y]
                                    .chars
                                    .push_str(&right_side.chars);
                            } else if editor.cursor.x > 0 {
                                editor.cursor.x -= 1;
                                editor.lines[editor.cursor.y].remove(editor.cursor.x);
                            }
                        }
                        Keycode::Delete => {
                            if editor.cursor.x == editor.lines[editor.cursor.y].chars.len()
                                && editor.lines.len() > editor.cursor.y + 1
                            {
                                let right_side = editor.lines.remove(editor.cursor.y + 1);
                                editor.lines[editor.cursor.y]
                                    .chars
                                    .push_str(&right_side.chars);
                            } else if editor.cursor.x < editor.lines[editor.cursor.y].chars.len() {
                                editor.lines[editor.cursor.y].remove(editor.cursor.x);
                            }
                        }
                        Keycode::Left if editor.cursor.x > 0 => editor.cursor.x -= 1,
                        Keycode::Right
                            if editor.cursor.x < editor.lines[editor.cursor.y].chars.len() =>
                        {
                            editor.cursor.x += 1
                        }
                        Keycode::Up if editor.cursor.y > 0 => {
                            editor.cursor.x = std::cmp::min(
                                editor.lines[editor.cursor.y - 1].chars.len(),
                                editor.cursor.x,
                            );
                            editor.cursor.y -= 1;
                        }
                        Keycode::Down if editor.cursor.y != editor.lines.len() - 1 => {
                            editor.cursor.x = std::cmp::min(
                                editor.lines[editor.cursor.y + 1].chars.len(),
                                editor.cursor.x,
                            );
                            editor.cursor.y += 1;
                        }
                        Keycode::Return => {
                            let new_line = editor.lines[editor.cursor.y]
                                .chars
                                .split_off(editor.cursor.x);
                            editor.cursor.x = 0;
                            editor.cursor.y += 1;
                            editor
                                .lines
                                .insert(editor.cursor.y, Line { chars: new_line });
                        }
                        _ => {}
                    },
                    _ => {}
                },
                Event::TextInput { text, .. } => {
                    editor.lines[editor.cursor.y].insert(&text, editor.cursor.x);
                    editor.cursor.x += text.len();
                }
                _ => {}
            }
        }

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();

        render_text(&mut canvas, &mut font, &editor, Color::WHITE, FONT_SCALE)?;
        render_cursor(&mut canvas, &mut font, &editor)?;

        canvas.present();
    }

    Ok(())
}
