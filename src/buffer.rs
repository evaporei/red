use crate::v2s;
use crate::vector::Vector2;

#[derive(Default)]
pub struct Line {
    pub chars: String,
}

impl Line {
    fn insert(&mut self, col: usize, text: &str) {
        self.chars.insert_str(col, text);
    }
    fn remove(&mut self, col: usize) {
        if !self.chars.is_empty() {
            self.chars.remove(col);
        }
    }
}

#[derive(Default)]
pub struct Buffer {
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

impl Buffer {
    pub fn new() -> Self {
        Self {
            filepath: None,
            lines: vec![Line::default()],
            cursor: v2s!(0),
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
        let mut buffer = Self::default();
        for line in io::BufReader::new(file).lines() {
            let mut chars = line?;
            if chars.ends_with('\n') {
                chars.pop();
            }
            buffer.lines.push(Line { chars });
        }
        if buffer.lines.is_empty() {
            buffer.lines.push(Line::default());
        }
        buffer.filepath = Some(filepath);
        Ok(buffer)
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
        self.lines[self.cursor.y].insert(self.cursor.x, text);
        self.cursor.x += text.len();
    }
    pub fn char_at_cursor(&self) -> Option<char> {
        self.lines[self.cursor.y].chars.chars().nth(self.cursor.x)
    }
}

pub struct Gap {
    pub(crate) buf: Vec<u8>,
    pub(crate) len: usize,
    pub(crate) start: usize,
    pub(crate) end: usize,
}

impl Gap {
    pub fn new(cap: usize) -> Self {
        Self {
            buf: vec![0; cap],
            len: 0,
            start: 0,
            end: cap,
        }
    }
    pub fn shift_gap_to(&mut self, cursor: usize) {
        let gap_len = self.end - self.start;
        let cursor = std::cmp::min(cursor, self.buf.capacity() - gap_len);
        if self.start == cursor {
            return;
        }
        if self.start < cursor {
            let delta = cursor - self.start;
            for i in 0..delta {
                self.buf[self.start + i] = self.buf[self.end + i];
            }
            self.start += delta;
            self.end += delta;
        } else {
            let delta = self.start - cursor;
            for i in 0..delta {
                self.buf[self.end - delta + i] = self.buf[self.start - delta + i];
            }
            self.start -= delta;
            self.end -= delta;
        }
    }

    pub fn grow(&mut self, n_required: usize) {
        let gap_len = self.end - self.start;
        if gap_len >= n_required {
            return;
        }
        self.shift_gap_to(self.buf.capacity() - gap_len);
        let new_cap = 2 * (n_required + self.buf.capacity() - gap_len);
        self.buf.reserve(new_cap - self.buf.capacity());
        self.buf
            .extend(std::iter::repeat(0).take(self.buf.capacity() - self.buf.len()));
        self.end = self.buf.capacity();
    }

    pub fn insert_char(&mut self, at: usize, ch: char) {
        let mut tmp: [u8; 4] = [0; 4];
        let s = ch.encode_utf8(&mut tmp);
        let s_bytes = s.as_bytes();
        self.grow(s_bytes.len());
        self.shift_gap_to(at);

        for (i, t) in s_bytes.iter().enumerate() {
            self.buf[self.start + i] = *t;
            self.len += 1;
        }
        self.start += s_bytes.len();
    }

    pub fn insert_str(&mut self, at: usize, s: &str) {
        let s_bytes = s.as_bytes();
        self.grow(s_bytes.len());
        self.shift_gap_to(at);
        for (i, b) in s_bytes.iter().enumerate() {
            self.buf[self.start + i] = *b;
            self.len += 1;
        }
        self.start += s_bytes.len();
    }

    pub fn to_str(&self) -> (&str, &str) {
        if self.len == 0 {
            return ("", "");
        }
        if self.start == 0 {
            return ("", unsafe {
                std::str::from_utf8_unchecked(&self.buf[self.end..])
            });
        }
        if self.end == self.buf.capacity() {
            return (
                unsafe { std::str::from_utf8_unchecked(&self.buf[..self.start]) },
                "",
            );
        }
        (
            unsafe { std::str::from_utf8_unchecked(&self.buf[..self.start]) },
            unsafe { std::str::from_utf8_unchecked(&self.buf[self.end..]) },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gap() {
        let mut g = Gap::new(16);
        assert_eq!(g.to_str(), ("", ""));
        assert_eq!(g.len, 0);
        assert_eq!(g.buf.capacity(), 16);
        assert_eq!(g.start, 0);
        assert_eq!(g.end, 16);
        g.insert_char(0, 'a');
        assert_eq!(g.len, 1);
        assert_eq!(g.buf.capacity(), 16);
        assert_eq!(g.start, 1);
        assert_eq!(g.end, 16);
        g.insert_str(1, "bcd");
        assert_eq!(g.len, 4);
        assert_eq!(g.buf.capacity(), 16);
        assert_eq!(g.start, 4);
        assert_eq!(g.end, 16);
        assert_eq!(g.to_str(), ("abcd", ""));
        g.shift_gap_to(0);
        assert_eq!(g.start, 0);
        assert_eq!(g.end, 12);
        g.insert_str(0, "xyz");
        assert_eq!(g.to_str(), ("xyz", "abcd"));
        g.insert_str(7, " this grows the buffer");
        assert_eq!(g.len, 29);
        assert_eq!(g.buf.capacity(), 58);
        assert_eq!(g.to_str(), ("xyzabcd this grows the buffer", ""));
    }
}
