mod engine;
mod game;
mod ui;
mod utils;

use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::{self, Result};

use ui::app::{App, GameState};

fn main() -> Result<()> {
    // terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // make the app, then run it
    let mut app = App::new();
    let res = run_app(&mut terminal, &mut app);

    // restore the terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}

fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> io::Result<()> {
    loop {
        terminal.draw(|f| app.draw(f))?;

        if app.should_quit {
            return Ok(());
        }

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('q') => {
                    app.should_quit = true;
                }
                KeyCode::Esc => match app.game_state {
                    GameState::Playing | GameState::About => {
                        app.game_state = GameState::Menu;
                        app.command_buffer.clear(); // get rid of any artifacts from previous screen when there is pending commands
                    }
                    GameState::Menu => {
                        app.should_quit = true;
                    }
                },
                KeyCode::Up => match app.game_state {
                    GameState::Menu => app.menu_index = app.menu_index.saturating_sub(1),
                    _ => {}
                },
                KeyCode::Down => match app.game_state {
                    GameState::Menu => app.menu_index = (app.menu_index + 1).min(1),
                    _ => {}
                },
                KeyCode::Enter => match app.game_state {
                    GameState::Menu => {
                        app.game_state = match app.menu_index {
                            0 => GameState::Playing,
                            1 => GameState::About,
                            _ => GameState::Menu,
                        };
                    }
                    GameState::Playing => {
                        if let Some(msg) = app.handle_command() {
                            println!("{}", msg);
                        }
                    }
                    GameState::About => app.game_state = GameState::Menu,
                },
                KeyCode::Char(c) => {
                    if let GameState::Playing = app.game_state {
                        app.command_buffer.push(c);
                    }
                }
                KeyCode::Backspace => {
                    if let GameState::Playing = app.game_state {
                        app.command_buffer.pop();
                    }
                }
                _ => {}
            }
        }
    }
}
