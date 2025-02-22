fn parse_coordinate(coord: &str) -> Option<(usize, usize)> {
    if coord.len() != 2 {
        return None;
    }

    let file = coord.chars().nth(0)?.to_ascii_lowercase();
    let rank = coord.chars().nth(1)?.to_digit(10)?;

    if !('a'..='h').contains(&file) || !(1..=8).contains(&rank) {
        return None;
    }

    let file_idx = (file as u8 - b'a') as usize;
    let rank_idx = 8 - rank as usize;

    Some((rank_idx, file_idx))
}

fn coordinate_to_string(pos: (usize, usize)) -> String {
    let file = (b'a' + pos.1 as u8) as char;
    let rank = 8 - pos.0;
    format!("{}{}", file, rank)
}
use crate::engine::rl::RLEngine;
use crate::game::board::Board;
use crate::game::piece::Color as PieceColor;
use crossterm::event::KeyCode;
use ratatui::{
    layout::{Constraint, Direction as LayoutDirection, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, Paragraph},
    Frame,
};

pub enum GameState {
    Menu,
    Playing,
    About,
}

pub struct App {
    pub game_state: GameState,
    pub board: Board,
    pub cursor_pos: (usize, usize),
    pub selected_piece: Option<(usize, usize)>,
    pub should_quit: bool,
    pub menu_index: usize,
    pub command_buffer: String,
    pub move_history: Vec<String>,
    pub history_scroll: usize,
    pub rl_engine: RLEngine,
    pub current_turn: PieceColor,
    pub bot_color: PieceColor,
    pub last_position_score: f32,
    pub current_position_score: f32,
}

impl App {
    pub fn new() -> Self {
        Self {
            game_state: GameState::Menu,
            board: Board::new(),
            cursor_pos: (0, 0),
            selected_piece: None,
            should_quit: false,
            menu_index: 0,
            command_buffer: String::new(),
            move_history: Vec::new(),
            history_scroll: 0,
            rl_engine: RLEngine::new(),
            current_turn: PieceColor::White,
            bot_color: PieceColor::Black,
            last_position_score: 0.0,
            current_position_score: 0.0,
        }
    }

    pub fn make_bot_move(&mut self) -> Option<String> {
        if self.current_turn == self.bot_color {
            if let Some((from, to)) = self.rl_engine.get_best_move(&self.board, self.bot_color) {
                let piece = self.board.get_piece(from).cloned();
                if let Some(piece) = piece {
                    if self.board.move_piece(from, to) {
                        let move_str = format!(
                            "{} {} → {}",
                            piece.to_char(),
                            coordinate_to_string(from),
                            coordinate_to_string(to)
                        );
                        self.move_history.push(move_str.clone());

                        // Update RL engine based on position evaluation
                        self.last_position_score = self.current_position_score;
                        self.current_position_score = self
                            .rl_engine
                            .evaluate_position(&self.board, self.bot_color);
                        self.rl_engine.update_position_values(
                            &self.board,
                            self.bot_color,
                            self.current_position_score,
                        );

                        // Switch turns
                        self.current_turn = PieceColor::White;
                        return Some("Bot moved successfully".to_string());
                    }
                }
            }
            Some("Bot failed to move".to_string())
        } else {
            None
        }
    }

    pub fn handle_command(&mut self) -> Option<String> {
        let cmd = self.command_buffer.trim().to_lowercase();
        let parts: Vec<&str> = cmd.split_whitespace().collect();

        if parts.len() == 2 {
            let from = parse_coordinate(parts[0]);
            let to = parse_coordinate(parts[1]);

            match (from, to) {
                (Some(from_pos), Some(to_pos)) => {
                    if let Some(piece) = self.board.get_piece(from_pos).cloned() {
                        if self.board.move_piece(from_pos, to_pos) {
                            let move_str = format!(
                                "{} {} → {}",
                                piece.to_char(),
                                coordinate_to_string(from_pos),
                                coordinate_to_string(to_pos)
                            );
                            self.move_history.push(move_str.clone());
                            self.command_buffer.clear();
                            // Switch turns after successful move
                            self.current_turn = self.bot_color;
                            let result = Some("Move successful".to_string());

                            // Trigger bot move if it's their turn
                            if let Some(bot_msg) = self.make_bot_move() {
                                self.move_history.push(format!("Bot: {}", bot_msg));
                            }

                            self.command_buffer.clear();
                            return result;
                        } else {
                            return Some("Invalid move".to_string());
                        }
                    } else {
                        return Some("No piece at selected position".to_string());
                    }
                }
                _ => return Some("Invalid coordinate format. Use a1-h8".to_string()),
            }
        } else {
            Some("Invalid command. Use: <from> <to> (e.g. 'e2 e4')".to_string())
        }
    }

    pub fn select_piece(&mut self) {
        let pos = self.cursor_pos;
        if let Some(_piece) = self.board.get_piece(pos) {
            if self.selected_piece.is_none() {
                // Select piece
                self.selected_piece = Some(pos);
            } else if self.selected_piece == Some(pos) {
                // Deselect piece
                self.selected_piece = None;
            } else {
                // Try to move selected piece to new position
                if let Some(from) = self.selected_piece {
                    if self.board.move_piece(from, pos) {
                        self.selected_piece = None;
                    }
                }
            }
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        match self.game_state {
            GameState::Menu => self.draw_menu(frame),
            GameState::Playing => self.draw_game(frame),
            GameState::About => self.draw_about(frame),
        }
    }

    fn draw_menu(&self, frame: &mut Frame) {
        let area = frame.area();

        let layout = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Length(8), // Title height
                Constraint::Length(3), // Spacing
                Constraint::Length(4), // Menu items
                Constraint::Min(0),
            ])
            .split(area);

        // Title
        let title = vec![
            Line::from("██████╗██╗  ██╗███████╗███████╗███████╗"),
            Line::from("██╔════╝██║  ██║██╔════╝██╔════╝██╔════╝"),
            Line::from("██║     ███████║█████╗  ███████╗███████╗"),
            Line::from("██║     ██╔══██║██╔══╝  ╚════██║╚════██║"),
            Line::from("╚██████╗██║  ██║███████╗███████║███████║"),
            Line::from("╚═════╝╚═╝  ╚═╝╚══════╝╚══════╝╚══════╝"),
        ];

        let title_block = Paragraph::new(title)
            .style(Style::default().fg(Color::Yellow))
            .alignment(ratatui::layout::Alignment::Center)
            .block(Block::default().borders(Borders::NONE));

        // Menu items
        let menu_items = vec![
            Line::from(vec![
                Span::styled("     ► ", Style::default().fg(Color::Reset)),
                Span::styled(
                    "PLAY",
                    Style::default()
                        .fg(if self.menu_index == 0 {
                            Color::Green
                        } else {
                            Color::White
                        })
                        .add_modifier(if self.menu_index == 0 {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                ),
            ]),
            Line::from(vec![
                Span::styled("     ► ", Style::default().fg(Color::White)),
                Span::styled(
                    "ABOUT",
                    Style::default()
                        .fg(if self.menu_index == 1 {
                            Color::Green
                        } else {
                            Color::White
                        })
                        .add_modifier(if self.menu_index == 1 {
                            Modifier::BOLD
                        } else {
                            Modifier::empty()
                        }),
                ),
            ]),
        ];

        let menu = Paragraph::new(menu_items)
            .alignment(ratatui::layout::Alignment::Left)
            .block(Block::default().borders(Borders::NONE));

        let instructions = Paragraph::new("Use ↑↓ arrows to select and ENTER to confirm")
            .style(Style::default().fg(Color::DarkGray))
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(title_block, layout[1]);
        frame.render_widget(menu, layout[3]);
        frame.render_widget(instructions, layout[4]);
    }

    fn draw_game(&self, frame: &mut Frame) {
        let area = frame.area();

        // Main horizontal split between game+analytics and history
        let main_layout = Layout::default()
            .direction(LayoutDirection::Horizontal)
            .constraints([
                Constraint::Ratio(3, 4), // Game board + analytics
                Constraint::Ratio(1, 4), // History + command
            ])
            .split(area);

        // Vertical split for board and analytics
        let left_layout = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([
                Constraint::Length(25), // Board height
                Constraint::Length(40), // Analytics height - increased from 15 to 40
            ])
            .split(main_layout[0]);

        // Vertical split for history and command input
        let right_layout = Layout::default()
            .direction(LayoutDirection::Vertical)
            .constraints([
                Constraint::Min(3),    // History takes most space
                Constraint::Length(3), // Command input height
            ])
            .split(main_layout[1]);

        let board_area = left_layout[0];

        // Create the board content
        let mut board_content = vec![];

        // Add column labels
        board_content.push(Line::from(vec![Span::raw(
            "     a    b    c    d    e    f    g    h",
        )]));

        // Add top border with vertical grid markers
        board_content.push(Line::from(Span::styled(
            "   ┌────┬────┬────┬────┬────┬────┬────┬────┐",
            Style::default().fg(Color::LightGreen),
        )));

        // Add board rows
        for rank in 0..8 {
            let mut row = vec![
                Span::styled(format!("{}  ", 8 - rank), Style::default().fg(Color::Green)),
                Span::styled("│ ", Style::default().fg(Color::Green)),
            ];
            for file in 0..8 {
                let _is_dark = (rank + file) % 2 == 1;
                let piece = self.board.get_piece((rank, file));
                let piece_char = piece.map_or(" ".to_string(), |p| p.to_char().to_string());

                let piece_color = if let Some(piece) = self.board.get_piece((rank, file)) {
                    if piece.color == crate::game::piece::Color::White {
                        Color::White
                    } else {
                        Color::Yellow
                    }
                } else {
                    Color::DarkGray
                };

                let style = Style::default().fg(piece_color);

                if (rank, file) == self.cursor_pos {
                    row.push(Span::styled(format!(" {}   ", piece_char), style));
                } else if Some((rank, file)) == self.selected_piece {
                    row.push(Span::styled(format!(" {}   ", piece_char), style));
                } else {
                    row.push(Span::styled(format!(" {}   ", piece_char), style));
                }
            }
            row.push(Span::styled(" │", Style::default().fg(Color::Green)));
            board_content.push(Line::from(row));

            // horizontal grid line after each row except the last
            if rank < 7 {
                let mut grid_line = vec![
                    Span::styled("   ", Style::default()),
                    Span::styled("├────", Style::default().fg(Color::LightGreen)),
                ];
                for _ in 0..7 {
                    grid_line.push(Span::styled(
                        "┼────",
                        Style::default().fg(Color::LightGreen),
                    ));
                }
                grid_line.push(Span::styled("┤", Style::default().fg(Color::LightGreen)));
                board_content.push(Line::from(grid_line));
            }

            // bottom border with vertical grid markers
            if rank == 7 {
                board_content.push(Line::from(Span::styled(
                    "   └────┴────┴────┴────┴────┴────┴────┴────┘",
                    Style::default().fg(Color::LightGreen),
                )));
            }
        }

        let board = Paragraph::new(board_content)
            .block(Block::default().borders(Borders::NONE))
            .style(Style::default().fg(Color::White));

        frame.render_widget(board, board_area);

        // analytics section
        let mut analytics_text = vec![
            Line::from(vec![
                Span::raw("Position Evaluation"),
                Span::raw(" ".repeat(30)),
                Span::raw("Engine Analysis"),
            ]),
            Line::from("─".repeat(60)),
            Line::from(vec![
                Span::raw("Thinking depth: "),
                Span::styled(
                    format!("{}", self.rl_engine.current_stats.depth_reached),
                    Style::default().fg(Color::Blue),
                ),
            ]),
            Line::from(vec![
                Span::raw("Total positions: "),
                Span::styled(
                    format!("{}", self.rl_engine.current_stats.total_simulations),
                    Style::default().fg(Color::Yellow),
                ),
            ]),
            Line::from(vec![
                Span::raw("Current Evaluation: "),
                Span::styled(
                    format!("{:.2}", self.rl_engine.current_stats.current_eval),
                    Style::default().fg(Color::Green),
                ),
            ]),
            Line::from(""),
            Line::from("Top Moves Considered:"),
        ];

        // add top moves
        for (idx, (mv, score, visits)) in self.rl_engine.current_stats.top_moves.iter().enumerate()
        {
            analytics_text.push(Line::from(vec![
                Span::raw(format!("{}. ", idx + 1)),
                Span::styled(mv.clone(), Style::default().fg(Color::Yellow)),
                Span::raw(" ("),
                Span::styled(format!("{:.2}", score), Style::default().fg(Color::Blue)),
                Span::raw(format!(", {} visits)", visits)),
            ]));
        }

        analytics_text.push(Line::from(""));
        analytics_text.push(Line::from(vec![
            Span::raw("Current Position Score: "),
            Span::styled(
                format!("{:.2}", self.current_position_score),
                Style::default().fg(Color::Green),
            ),
        ]));
        analytics_text.push(Line::from(""));
        analytics_text.push(Line::from(vec![
            Span::raw("Material Balance: "),
            Span::styled(
                format!(
                    "{}",
                    self.rl_engine
                        .get_material_balance(&self.board, self.bot_color)
                ),
                Style::default().fg(Color::Blue),
            ),
        ]));
        analytics_text.push(Line::from(vec![
            Span::raw("King Safety: "),
            Span::styled(
                format!(
                    "{:.2}",
                    self.rl_engine.get_king_safety(&self.board, self.bot_color)
                ),
                Style::default().fg(Color::Magenta),
            ),
        ]));
        analytics_text.push(Line::from(vec![
            Span::raw("Center Control: "),
            Span::styled(
                format!(
                    "{:.2}",
                    self.rl_engine
                        .get_center_control(&self.board, self.bot_color)
                ),
                Style::default().fg(Color::Cyan),
            ),
        ]));

        let analytics = Paragraph::new(analytics_text)
            .block(Block::default().borders(Borders::ALL).title("Analytics"))
            .style(Style::default().fg(Color::White));

        frame.render_widget(analytics, left_layout[1]);

        // Move history on right side
        let visible_history: Vec<&str> = self
            .move_history
            .iter()
            .skip(self.history_scroll)
            .map(|s| s.as_str())
            .collect();

        let history = Paragraph::new(visible_history.join("\n"))
            .block(Block::default().borders(Borders::ALL).title("Move History"))
            .style(Style::default().fg(Color::White))
            .scroll((self.history_scroll as u16, 0));

        frame.render_widget(history, right_layout[0]);

        // command input at bottom
        let input = Paragraph::new(format!(">> {}", self.command_buffer))
            .block(Block::default().borders(Borders::ALL).title("Command"))
            .style(Style::default().fg(Color::Yellow));

        frame.render_widget(input, right_layout[1]);
    }

    pub fn scroll_history(&mut self, up: bool) {
        if up {
            if self.history_scroll > 0 {
                self.history_scroll -= 1;
            }
        } else {
            if self.history_scroll < self.move_history.len().saturating_sub(1) {
                self.history_scroll += 1;
            }
        }
    }

    pub fn handle_key_event(&mut self, key_event: crossterm::event::KeyEvent) {
        match key_event.code {
            KeyCode::Up => self.scroll_history(true),
            KeyCode::Down => self.scroll_history(false),
            KeyCode::PageUp => {
                for _ in 0..5 {
                    self.scroll_history(true);
                }
            }
            KeyCode::PageDown => {
                for _ in 0..5 {
                    self.scroll_history(false);
                }
            }
            _ => {}
        }
    }

    fn draw_about(&self, frame: &mut Frame) {
        let area = frame.area();

        let about_text = vec![
            Line::from("ChessRL"),
            Line::from("-------------------"),
            Line::from(""),
            Line::from("chess game with ML opponent"),
            Line::from("featuring pure reinforcement learning."),
            Line::from(""),
            Line::from("Commands:"),
            Line::from("e2 e4  - Move a piece from e2 to e4"),
            Line::from("ESC - Return to menu"),
            Line::from("Q   - Quit game"),
            Line::from(""),
            Line::from("The AI learns as the game is played,"),
            Line::from("improving its strategy over time."),
            Line::from(""),
            Line::from("Created by @frgmt0"),
        ];

        let about_block = Paragraph::new(about_text)
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL).title("About"))
            .alignment(ratatui::layout::Alignment::Center);

        frame.render_widget(Clear, area);
        frame.render_widget(about_block, area);
    }
}
