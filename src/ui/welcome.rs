use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    style::{Color, Print, SetForegroundColor},
    terminal::{Clear, ClearType},
};
use std::io::{stdout, Write};

#[derive(Clone)]
pub enum MenuItem {
    Play,
    About,
}

pub struct WelcomeScreen {
    selected_item: MenuItem,
}

impl WelcomeScreen {
    pub fn new() -> Self {
        WelcomeScreen {
            selected_item: MenuItem::Play,
        }
    }

    pub fn display(&self) -> std::io::Result<()> {
        let mut stdout = stdout();
        execute!(stdout, Clear(ClearType::All))?;

        let title = r#"

             ██████╗██╗  ██╗███████╗███████╗███████╗
             ██╔════╝██║  ██║██╔════╝██╔════╝██╔════╝
             ██║     ███████║█████╗  ███████╗███████╗
             ██║     ██╔══██║██╔══╝  ╚════██║╚════██║
             ╚██████╗██║  ██║███████╗███████║███████║
             ╚═════╝╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝

"#;

        execute!(
            stdout,
            SetForegroundColor(Color::Yellow),
            Print(title),
            SetForegroundColor(Color::Reset)
        )?;

        // items
        let play_color = if matches!(self.selected_item, MenuItem::Play) {
            Color::Green
        } else {
            Color::White
        };
        let about_color = if matches!(self.selected_item, MenuItem::About) {
            Color::Green
        } else {
            Color::White
        };

        execute!(
            stdout,
            Print("\n\n"),
            SetForegroundColor(play_color),
            Print("     ► PLAY\n"),
            SetForegroundColor(about_color),
            Print("     ► ABOUT\n"),
            SetForegroundColor(Color::Reset),
            Print("\n\n     Use ↑↓ arrows to select and ENTER to confirm\n")
        )?;

        stdout.flush()?;
        Ok(())
    }

    pub fn handle_input(&mut self) -> Option<MenuItem> {
        if let Ok(Event::Key(key_event)) = event::read() {
            match key_event.code {
                KeyCode::Up | KeyCode::Char('w') => {
                    self.selected_item = MenuItem::Play;
                    None
                }
                KeyCode::Down | KeyCode::Char('s') => {
                    self.selected_item = MenuItem::About;
                    None
                }
                KeyCode::Enter => Some(self.selected_item.clone()),
                _ => None,
            }
        } else {
            None
        }
    }
    // old about screen. look in app.rs for the new one
    pub fn show_about(&self) -> std::io::Result<()> {
        let mut stdout = stdout();
        execute!(stdout, Clear(ClearType::All))?;

        let about_text = r#"
    ♜ ♞ ♝ ♛ ♚ ♝ ♞ ♜
    ♟ ♟ ♟ ♟ ♟ ♟ ♟ ♟

    Terminal Chess Game
    ------------------
    A simple chess implementation
    with reinforcement learning capabilities.

    Created with ♥ in Rust

    Press ESC to return to menu
    Press Q to quit game
    "#;

        execute!(
            stdout,
            SetForegroundColor(Color::Cyan),
            Print(about_text),
            SetForegroundColor(Color::Reset)
        )?;

        stdout.flush()?;
        Ok(())
    }
}
