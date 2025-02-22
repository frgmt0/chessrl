use crossterm::{
    cursor::{Hide, Show},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, size},
};
use std::io::stdout;

#[derive(Debug)]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

#[derive(Debug)]
pub enum UserAction {
    Move(Direction),
    Select,
    Quit,
}

pub struct TerminalUI {
    cursor_pos: (usize, usize),
    terminal_size: (u16, u16),
}

impl TerminalUI {
    pub fn new() -> Self {
        let terminal_size = size().unwrap_or((80, 24));
        Self {
            cursor_pos: (0, 0),
            terminal_size,
        }
    }

    pub fn get_terminal_size(&self) -> (u16, u16) {
        self.terminal_size
    }

    pub fn init() -> std::io::Result<()> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, Hide)?;
        Ok(())
    }

    pub fn cleanup() -> std::io::Result<()> {
        let mut stdout = stdout();
        execute!(stdout, Show)?;
        disable_raw_mode()?;
        Ok(())
    }

    pub fn move_cursor(&mut self, direction: Direction) {
        match direction {
            Direction::Up if self.cursor_pos.1 > 0 => self.cursor_pos.1 -= 1,
            Direction::Down if self.cursor_pos.1 < 7 => self.cursor_pos.1 += 1,
            Direction::Left if self.cursor_pos.0 > 0 => self.cursor_pos.0 -= 1,
            Direction::Right if self.cursor_pos.0 < 7 => self.cursor_pos.0 += 1,
            _ => {}
        }
    }

    pub fn get_cursor_pos(&self) -> (usize, usize) {
        self.cursor_pos
    }
}
