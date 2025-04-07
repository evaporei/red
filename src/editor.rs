use crate::vector2::Vector2;

#[derive(Default)]
pub struct Line {
    pub chars: String,
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
pub struct Editor {
    filepath: Option<PathBuf>,
    pub lines: Vec<Line>,
    pub cursor: Vector2<usize>,
}

use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::Write;
use std::path::PathBuf;
use std::str::FromStr;

impl Editor {
    pub fn new() -> Self {
        Self {
            filepath: None,
            lines: vec![Line::default()],
            cursor: Vector2::new(0, 0),
        }
    }
    pub fn from_filepath(filepath: String) -> std::io::Result<Self> {
        let filepath = PathBuf::from(filepath);
        let file = match File::open(&filepath) {
            Ok(file) => file,
            Err(_) => {
                // it's alright if file doesn't exist
                return Ok(Self {
                    filepath: Some(filepath),
                    ..Self::new()
                });
            }
        };
        let mut editor = Self::default();
        for line in io::BufReader::new(file).lines() {
            let mut chars = line?;
            if chars.ends_with('\n') {
                chars.pop();
            }
            editor.lines.push(Line { chars });
        }
        if editor.lines.is_empty() {
            editor.lines.push(Line::default());
        }
        editor.filepath = Some(filepath);
        Ok(editor)
    }
    pub fn save(&self) -> std::io::Result<()> {
        let mut file = std::fs::File::options()
            .create(true)
            .write(true)
            .truncate(true)
            .open(
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
    pub fn backspace(&mut self) {
        if self.cursor.x == 0 && self.cursor.y > 0 {
            let right_side = self.lines.remove(self.cursor.y);
            self.cursor.y -= 1;
            self.cursor.x = self.lines[self.cursor.y].chars.len();
            self.lines[self.cursor.y].chars.push_str(&right_side.chars);
        } else if self.cursor.x > 0 {
            self.cursor.x -= 1;
            self.lines[self.cursor.y].remove(self.cursor.x);
        }
    }
    pub fn delete(&mut self) {
        if self.cursor.x == self.lines[self.cursor.y].chars.len()
            && self.lines.len() > self.cursor.y + 1
        {
            let right_side = self.lines.remove(self.cursor.y + 1);
            self.lines[self.cursor.y].chars.push_str(&right_side.chars);
        } else if self.cursor.x < self.lines[self.cursor.y].chars.len() {
            self.lines[self.cursor.y].remove(self.cursor.x);
        }
    }
    pub fn move_left(&mut self) {
        if self.cursor.x > 0 {
            self.cursor.x -= 1
        }
    }
    pub fn move_right(&mut self) {
        if self.cursor.x < self.lines[self.cursor.y].chars.len() {
            self.cursor.x += 1;
        }
    }
    pub fn move_up(&mut self) {
        if self.cursor.y > 0 {
            self.cursor.x = std::cmp::min(self.lines[self.cursor.y - 1].chars.len(), self.cursor.x);
            self.cursor.y -= 1;
        }
    }
    pub fn move_down(&mut self) {
        if self.cursor.y != self.lines.len() - 1 {
            self.cursor.x = std::cmp::min(self.lines[self.cursor.y + 1].chars.len(), self.cursor.x);
            self.cursor.y += 1;
        }
    }
    pub fn newline(&mut self) {
        let new_line = self.lines[self.cursor.y].chars.split_off(self.cursor.x);
        self.cursor.x = 0;
        self.cursor.y += 1;
        self.lines.insert(self.cursor.y, Line { chars: new_line });
    }
    pub fn insert_text(&mut self, text: &str) {
        self.lines[self.cursor.y].insert(text, self.cursor.x);
        self.cursor.x += text.len();
    }
}
