use crate::game::piece::{Color, Piece, PieceType};

#[derive(Clone)]
pub struct Board {
    squares: [[Option<Piece>; 8]; 8],
    selected_square: Option<(usize, usize)>,
    current_turn: Color,
}

impl Board {
    pub fn new() -> Self {
        let mut board = Board {
            squares: [[None; 8]; 8],
            selected_square: None,
            current_turn: Color::White,
        };
        board.initialize_pieces();
        board
    }

    fn initialize_pieces(&mut self) {
        // back rank pieces
        let back_rank = [
            PieceType::Rook,
            PieceType::Knight,
            PieceType::Bishop,
            PieceType::Queen,
            PieceType::King,
            PieceType::Bishop,
            PieceType::Knight,
            PieceType::Rook,
        ];

        // set up white pieces
        for (i, &piece_type) in back_rank.iter().enumerate() {
            self.squares[7][i] = Some(Piece::new(piece_type, Color::White));
            self.squares[6][i] = Some(Piece::new(PieceType::Pawn, Color::White));
        }

        // set up black pieces
        for (i, &piece_type) in back_rank.iter().enumerate() {
            self.squares[0][i] = Some(Piece::new(piece_type, Color::Black));
            self.squares[1][i] = Some(Piece::new(PieceType::Pawn, Color::Black));
        }
    }

    pub fn display(&self, _cursor_pos: (usize, usize), term_size: (u16, u16)) {
        print!("\x1B[2J\x1B[1;1H");

        // vertical padding to center the board
        let board_height = 10; // 8 ranks + 2 border lines
        let _v_padding = ((term_size.1 as i32 - board_height as i32) / 2).max(0) as u16;
        let _h_padding = ((term_size.0 as i32 - 35) / 2).max(0) as u16; // 35 is the new width of the board

        // i got rid of this method kinda. i left it in case i need it later but i moved to the ratatui rendering
    }

    pub fn get_piece(&self, pos: (usize, usize)) -> Option<&Piece> {
        self.squares[pos.0][pos.1].as_ref()
    }

    pub fn move_piece(&mut self, from: (usize, usize), to: (usize, usize)) -> bool {
        // basic validation
        if from == to {
            return false;
        }

        // check is there's a piece first
        let piece = match self.squares[from.0][from.1] {
            Some(p) => p,
            None => return false,
        };

        // check if the destination contains a piece of the same color and reject it if true
        if let Some(dest_piece) = self.squares[to.0][to.1] {
            if dest_piece.color == piece.color {
                return false;
            }
        }

        // validate piece-specific movement
        let valid = match piece.piece_type {
            PieceType::Pawn => self.validate_pawn_move(from, to, piece.color),
            PieceType::Rook => self.validate_rook_move(from, to),
            PieceType::Knight => self.validate_knight_move(from, to),
            PieceType::Bishop => self.validate_bishop_move(from, to),
            PieceType::Queen => self.validate_queen_move(from, to),
            PieceType::King => self.validate_king_move(from, to),
        };

        if !valid {
            return false;
        }

        //  else move the piece
        self.squares[from.0][from.1] = None;
        self.squares[to.0][to.1] = Some(piece);
        true
    }

    fn validate_pawn_move(&self, from: (usize, usize), to: (usize, usize), color: Color) -> bool {
        let direction = if color == Color::White { -1i8 } else { 1i8 };
        let start_rank = if color == Color::White { 6 } else { 1 };

        // convert to signed for safe arithmetic
        let from_rank = from.0 as i8;
        let from_file = from.1 as i8;
        let to_rank = to.0 as i8;
        let to_file = to.1 as i8;

        // pawn moves
        if from_file == to_file {
            // move forward
            if to_rank == from_rank + direction && self.squares[to.0][to.1].is_none() {
                return true;
            }
            // double move from starting position only
            if from.0 == start_rank
                && to_rank == from_rank + (2 * direction)
                && self.squares[to.0][to.1].is_none()
                && self.squares[(from_rank + direction) as usize][from.1].is_none()
            {
                return true;
            }
        }
        // capture moves (the diagonals)
        else if (to_file == from_file - 1 || to_file == from_file + 1)
            && to_rank == from_rank + direction
        {
            return self.squares[to.0][to.1].is_some();
        }

        false
    }

    fn validate_rook_move(&self, from: (usize, usize), to: (usize, usize)) -> bool {
        // rook moves horizontally or vertically but not diagonally
        if from.0 != to.0 && from.1 != to.1 {
            return false;
        }

        // check path is clear or return invalid move
        let rank_step = (to.0 as i8 - from.0 as i8).signum() as i8;
        let file_step = (to.1 as i8 - from.1 as i8).signum() as i8;

        let mut current = (from.0 as i8 + rank_step, from.1 as i8 + file_step);
        let to = (to.0 as i8, to.1 as i8);

        while (current.0, current.1) != (to.0, to.1) {
            if self.squares[current.0 as usize][current.1 as usize].is_some() {
                return false;
            }
            current = (current.0 + rank_step, current.1 + file_step);
        }

        true
    }

    fn validate_knight_move(&self, from: (usize, usize), to: (usize, usize)) -> bool {
        let rank_diff = (to.0 as i8 - from.0 as i8).abs();
        let file_diff = (to.1 as i8 - from.1 as i8).abs();

        (rank_diff == 2 && file_diff == 1) || (rank_diff == 1 && file_diff == 2)
    }

    fn validate_bishop_move(&self, from: (usize, usize), to: (usize, usize)) -> bool {
        let rank_diff = (to.0 as i8 - from.0 as i8).abs();
        let file_diff = (to.1 as i8 - from.1 as i8).abs();

        // bishop moves diagonally
        if rank_diff != file_diff {
            return false;
        }

        // check path is clear first
        let rank_step = (to.0 as i8 - from.0 as i8).signum();
        let file_step = (to.1 as i8 - from.1 as i8).signum();

        let mut current = (from.0 as i8 + rank_step, from.1 as i8 + file_step);
        let to = (to.0 as i8, to.1 as i8);

        while (current.0, current.1) != (to.0, to.1) {
            if self.squares[current.0 as usize][current.1 as usize].is_some() {
                return false;
            }
            current = (current.0 + rank_step, current.1 + file_step);
        }

        true
    }

    fn validate_queen_move(&self, from: (usize, usize), to: (usize, usize)) -> bool {
        self.validate_rook_move(from, to) || self.validate_bishop_move(from, to)
    }

    fn validate_king_move(&self, from: (usize, usize), to: (usize, usize)) -> bool {
        let rank_diff = (to.0 as i8 - from.0 as i8).abs();
        let file_diff = (to.1 as i8 - from.1 as i8).abs();

        rank_diff <= 1 && file_diff <= 1
    }
}
