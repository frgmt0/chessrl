use std::collections::HashMap;
use rand::Rng;
use crate::game::{
    board::Board,
    piece::{Color, PieceType},
};
use crate::utils::coordinate_to_string;

const MAX_PLIES: i32 = 10;
const MAX_OPPONENT_MOVES: usize = 150;
const UCT_CONSTANT: f32 = 1.414;

struct MCTSNode {
    board: Board,
    visits: u32,
    total_value: f32,
    children: Vec<(((usize, usize), (usize, usize)), MCTSNode)>,
    unexplored_moves: Vec<((usize, usize), (usize, usize))>,
    current_player: Color,
}

impl MCTSNode {
    fn new(board: Board, current_player: Color, engine: &RLEngine) -> Self {
        let moves = engine.generate_ranked_moves(&board, current_player);
        MCTSNode {
            board,
            visits: 0,
            total_value: 0.0,
            children: Vec::new(),
            unexplored_moves: moves,
            current_player,
        }
    }

    fn uct_value(&self, parent_visits: u32) -> f32 {
        if self.visits == 0 {
            return f32::INFINITY;
        }
        let exploitation = self.total_value / self.visits as f32;
        let exploration = UCT_CONSTANT * ((parent_visits as f32).ln() / self.visits as f32).sqrt();
        exploitation + exploration
    }
}

#[derive(Default, Clone)]
pub struct SimulationStats {
    pub total_simulations: u32,
    pub nodes_explored: u32,
    pub best_line: Vec<String>,
    pub best_move_confidence: f32,
    pub current_eval: f32,
    pub depth_reached: i32,
    pub top_moves: Vec<(String, f32, u32)>, // (move, score, visits)
    pub thinking_line: String,
}

impl SimulationStats {
    fn new() -> Self {
        SimulationStats {
            total_simulations: 0,
            nodes_explored: 0,
            best_line: Vec::new(),
            best_move_confidence: 0.0,
            current_eval: 0.0,
            depth_reached: 0,
            top_moves: Vec::new(),
            thinking_line: String::new(),
        }
    }
}

pub struct RLEngine {
    piece_values: HashMap<PieceType, i32>,
    position_values: HashMap<PieceType, [[f32; 8]; 8]>,
    learning_rate: f32,
    discount_factor: f32,
    exploration_rate: f32,
    move_history: Vec<((usize, usize), (usize, usize))>,
    simulation_depth: i32,
    prune_threshold: f32,
    pub current_stats: SimulationStats,
}

struct BoardAnalysis {
    controlled_squares: [[bool; 8]; 8],
    piece_mobility: HashMap<(usize, usize), Vec<(usize, usize)>>,
    threats: Vec<((usize, usize), (usize, usize))>,
    king_safety: f32,
    material_balance: i32,
    center_control: f32,
}

impl RLEngine {
    pub fn new() -> Self {
        let mut piece_values = std::collections::HashMap::new();
        piece_values.insert(PieceType::Pawn, 100);
        piece_values.insert(PieceType::Knight, 320);
        piece_values.insert(PieceType::Bishop, 330);
        piece_values.insert(PieceType::Rook, 500);
        piece_values.insert(PieceType::Queen, 900);
        piece_values.insert(PieceType::King, 20000);

        RLEngine {
            piece_values,
            position_values: Self::initialize_position_values(),
            learning_rate: 0.1,
            discount_factor: 0.95,
            exploration_rate: 0.1,
            move_history: Vec::new(),
            simulation_depth: MAX_PLIES,
            prune_threshold: -500.0,
            current_stats: SimulationStats::new(),
        }
    }


    fn initialize_position_values() -> std::collections::HashMap<PieceType, [[f32; 8]; 8]> {
        let mut values = std::collections::HashMap::new();
        
        // Initialize basic position values for each piece type
        // These will be updated through learning
        values.insert(PieceType::Pawn, Self::pawn_position_values());
        values.insert(PieceType::Knight, Self::knight_position_values());
        values.insert(PieceType::Bishop, Self::bishop_position_values());
        values.insert(PieceType::Rook, Self::rook_position_values());
        values.insert(PieceType::Queen, Self::queen_position_values());
        values.insert(PieceType::King, Self::king_position_values());
        
        values
    }

    pub fn update_position_values(&mut self, board: &Board, _color: Color, reward: f32) {
        // Update position values based on reward
        for rank in 0..8 {
            for file in 0..8 {
                if let Some(piece) = board.get_piece((rank, file)) {
                    let current_value = self.position_values.get_mut(&piece.piece_type).unwrap()[rank][file];
                    let update = self.learning_rate * (reward - current_value);
                    self.position_values.get_mut(&piece.piece_type).unwrap()[rank][file] += update;
                }
            }
        }
    }

    pub fn get_material_balance(&self, board: &Board, color: Color) -> i32 {
        let mut balance = 0;
        for rank in 0..8 {
            for file in 0..8 {
                if let Some(piece) = board.get_piece((rank, file)) {
                    let value = self.piece_values[&piece.piece_type];
                    if piece.color == color {
                        balance += value;
                    } else {
                        balance -= value;
                    }
                }
            }
        }
        balance
    }

    pub fn get_king_safety(&self, board: &Board, color: Color) -> f32 {
        if let Some(king_pos) = self.find_king(board, color) {
            let analysis = self.analyze_board(board, color);
            self.evaluate_king_safety(board, king_pos, color, &analysis)
        } else {
            0.0
        }
    }

    pub fn get_center_control(&self, board: &Board, color: Color) -> f32 {
        let analysis = self.analyze_board(board, color);
        self.evaluate_center_control(&analysis.controlled_squares)
    }

    pub fn evaluate_position(&self, board: &Board, color: Color) -> f32 {
        let analysis = self.analyze_board(board, color);
        let opponent_analysis = self.analyze_board(board, color.opposite());

        // Base score from material and position
        let mut score = analysis.material_balance as f32;

        // King safety (heavily weighted)
        score += analysis.king_safety * 3.0;
        score -= opponent_analysis.king_safety * 2.5;

        // Mobility bonus
        score += (analysis.piece_mobility.values().map(|moves| moves.len()).sum::<usize>() as f32) * 0.1;

        // Threat penalty
        score -= (analysis.threats.len() as f32) * 2.0;

        // Center control
        score += analysis.center_control * 1.5;

        // Randomization factor to avoid repetitive play
        let mut rng = rand::thread_rng();
        score += rng.gen_range(-0.2..0.2);

        score
    }

    pub fn get_best_move(&mut self, board: &Board, color: Color) -> Option<((usize, usize), (usize, usize))> {
        self.current_stats = SimulationStats::default();
        let mut root = MCTSNode::new(board.clone(), color, self);
        let start_time = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(5);

        while start_time.elapsed() < timeout {
            self.current_stats.total_simulations += 1;
            let eval = self.mcts_iteration(&mut root);
            
            // Update stats every 50 simulations
            if self.current_stats.total_simulations % 50 == 0 {
                self.current_stats.current_eval = eval;
                
                // Update top moves
                let mut top_moves = Vec::new();
                for (mv, child) in &root.children {
                    let score = child.total_value / child.visits as f32;
                    let move_str = format!("{}{}", 
                        coordinate_to_string(mv.0),
                        coordinate_to_string(mv.1)
                    );
                    top_moves.push((move_str, score, child.visits));
                }
                
                // Sort by visits and take top 3
                top_moves.sort_by(|a, b| b.2.cmp(&a.2));
                top_moves.truncate(3);
                self.current_stats.top_moves = top_moves;
                
                // Force UI refresh through crossterm
                let _ = crossterm::execute!(
                    std::io::stdout(),
                    crossterm::cursor::Hide,
                );
                let backend = ratatui::backend::CrosstermBackend::new(std::io::stdout());
                if let Ok(mut terminal) = ratatui::Terminal::new(backend) {
                    let _ = terminal.draw(|f| {
                        // This will be handled by the App's draw method
                        f.render_widget(ratatui::widgets::Clear, f.area());
                    });
                }
            }
        }

        // Find best child and record statistics
        if let Some((best_move, best_child)) = root.children.iter()
            .max_by_key(|(_, child)| child.visits) {
                
            // Calculate confidence as visits ratio
            let total_visits: u32 = root.children.iter()
                .map(|(_, child)| child.visits)
                .sum();
            self.current_stats.best_move_confidence = best_child.visits as f32 / total_visits as f32;
            
            // Record best line
            self.current_stats.best_line = vec![
                format!("{}{}", 
                    coordinate_to_string(best_move.0),
                    coordinate_to_string(best_move.1)
                )
            ];
            
            Some(*best_move)
        } else {
            None
        }
    }

    fn mcts_iteration(&mut self, node: &mut MCTSNode) -> f32 {
        self.current_stats.nodes_explored += 1;
        if node.visits > 0 && !node.unexplored_moves.is_empty() {
            let move_index = rand::thread_rng().gen_range(0..node.unexplored_moves.len());
            let next_move = node.unexplored_moves.remove(move_index);
            let mut new_board = node.board.clone();
            
            if new_board.move_piece(next_move.0, next_move.1) {
                let mut child = MCTSNode::new(new_board, node.current_player.opposite(), self);
                let value = -self.simulate(&mut child, self.simulation_depth);
                child.visits = 1;
                child.total_value = value;
                node.children.push((next_move, child));
                node.visits += 1;
                node.total_value += value;
                return value;
            }
            return self.mcts_iteration(node);
        }

        if node.children.is_empty() {
            let value = self.evaluate_position(&node.board, node.current_player);
            node.visits += 1;
            node.total_value += value;
            return value;
        }

        let parent_visits = node.visits;
        let (_, child) = node.children.iter_mut()
            .max_by(|(_, a), (_, b)| {
                a.uct_value(parent_visits)
                    .partial_cmp(&b.uct_value(parent_visits))
                    .unwrap()
            })
            .unwrap();

        let value = -self.mcts_iteration(child);
        node.visits += 1;
        node.total_value += value;
        value
    }

    fn simulate(&self, node: &mut MCTSNode, depth: i32) -> f32 {
        if depth <= 0 || self.is_terminal(&node.board) {
            return self.evaluate_position(&node.board, node.current_player);
        }

        let moves = self.generate_ranked_moves(&node.board, node.current_player);
        if moves.is_empty() {
            return self.evaluate_position(&node.board, node.current_player);
        }

        let num_moves = moves.len().min(MAX_OPPONENT_MOVES);
        let move_index = rand::thread_rng().gen_range(0..num_moves);
        let (from, to) = moves[move_index];

        let mut new_board = node.board.clone();
        if new_board.move_piece(from, to) {
            let mut child = MCTSNode::new(new_board, node.current_player.opposite(), self);
            -self.simulate(&mut child, depth - 1)
        } else {
            self.evaluate_position(&node.board, node.current_player)
        }
    }

    fn generate_ranked_moves(&self, board: &Board, color: Color) -> Vec<((usize, usize), (usize, usize))> {
        let mut moves = Vec::new();
        let analysis = self.analyze_board(board, color);

        for rank in 0..8 {
            for file in 0..8 {
                let from = (rank, file);
                if let Some(piece) = board.get_piece(from) {
                    if piece.color == color {
                        if let Some(possible_moves) = analysis.piece_mobility.get(&from) {
                            for &to in possible_moves {
                                let mut board_copy = board.clone();
                                if board_copy.move_piece(from, to) {
                                    if !self.is_king_threatened(&board_copy, color) {
                                        let score = self.evaluate_move_priority(board, from, to, &analysis);
                                        moves.push((from, to, score));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        moves.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        moves.truncate(MAX_OPPONENT_MOVES);
        moves.into_iter().map(|(from, to, _)| (from, to)).collect()
    }

    fn evaluate_move_priority(&self, board: &Board, from: (usize, usize), to: (usize, usize), analysis: &BoardAnalysis) -> f32 {
        let mut priority = 0.0;

        if let Some(target) = board.get_piece(to) {
            priority += self.piece_values[&target.piece_type] as f32;
        }

        if analysis.threats.iter().any(|(_, target)| *target == from) {
            priority += 50.0;
        }

        if (to.0 >= 3 && to.0 <= 4) && (to.1 >= 3 && to.1 <= 4) {
            priority += 10.0;
        }

        priority
    }

    fn is_terminal(&self, _board: &Board) -> bool {
        false
    }

    // Position value matrices for each piece type
    fn pawn_position_values() -> [[f32; 8]; 8] {
        [
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
            [5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0],
            [1.0, 1.0, 2.0, 3.0, 3.0, 2.0, 1.0, 1.0],
            [0.5, 0.5, 1.0, 2.5, 2.5, 1.0, 0.5, 0.5],
            [0.0, 0.0, 0.0, 2.0, 2.0, 0.0, 0.0, 0.0],
            [0.5, -0.5, -1.0, 0.0, 0.0, -1.0, -0.5, 0.5],
            [0.5, 1.0, 1.0, -2.0, -2.0, 1.0, 1.0, 0.5],
            [0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        ]
    }

    fn knight_position_values() -> [[f32; 8]; 8] {
        [
            [-5.0, -4.0, -3.0, -3.0, -3.0, -3.0, -4.0, -5.0],
            [-4.0, -2.0,  0.0,  0.0,  0.0,  0.0, -2.0, -4.0],
            [-3.0,  0.0,  1.0,  1.5,  1.5,  1.0,  0.0, -3.0],
            [-3.0,  0.5,  1.5,  2.0,  2.0,  1.5,  0.5, -3.0],
            [-3.0,  0.0,  1.5,  2.0,  2.0,  1.5,  0.0, -3.0],
            [-3.0,  0.5,  1.0,  1.5,  1.5,  1.0,  0.5, -3.0],
            [-4.0, -2.0,  0.0,  0.5,  0.5,  0.0, -2.0, -4.0],
            [-5.0, -4.0, -3.0, -3.0, -3.0, -3.0, -4.0, -5.0],
        ]
    }

    fn bishop_position_values() -> [[f32; 8]; 8] {
        [
            [-2.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -2.0],
            [-1.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -1.0],
            [-1.0,  0.0,  0.5,  1.0,  1.0,  0.5,  0.0, -1.0],
            [-1.0,  0.5,  0.5,  1.0,  1.0,  0.5,  0.5, -1.0],
            [-1.0,  0.0,  1.0,  1.0,  1.0,  1.0,  0.0, -1.0],
            [-1.0,  1.0,  1.0,  1.0,  1.0,  1.0,  1.0, -1.0],
            [-1.0,  0.5,  0.0,  0.0,  0.0,  0.0,  0.5, -1.0],
            [-2.0, -1.0, -1.0, -1.0, -1.0, -1.0, -1.0, -2.0],
        ]
    }

    fn rook_position_values() -> [[f32; 8]; 8] {
        [
            [ 0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0],
            [ 0.5,  1.0,  1.0,  1.0,  1.0,  1.0,  1.0,  0.5],
            [-0.5,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -0.5],
            [-0.5,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -0.5],
            [-0.5,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -0.5],
            [-0.5,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -0.5],
            [-0.5,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -0.5],
            [ 0.0,  0.0,  0.0,  0.5,  0.5,  0.0,  0.0,  0.0],
        ]
    }

    fn queen_position_values() -> [[f32; 8]; 8] {
        [
            [-2.0, -1.0, -1.0, -0.5, -0.5, -1.0, -1.0, -2.0],
            [-1.0,  0.0,  0.0,  0.0,  0.0,  0.0,  0.0, -1.0],
            [-1.0,  0.0,  0.5,  0.5,  0.5,  0.5,  0.0, -1.0],
            [-0.5,  0.0,  0.5,  0.5,  0.5,  0.5,  0.0, -0.5],
            [ 0.0,  0.0,  0.5,  0.5,  0.5,  0.5,  0.0, -0.5],
            [-1.0,  0.5,  0.5,  0.5,  0.5,  0.5,  0.0, -1.0],
            [-1.0,  0.0,  0.5,  0.0,  0.0,  0.0,  0.0, -1.0],
            [-2.0, -1.0, -1.0, -0.5, -0.5, -1.0, -1.0, -2.0],
        ]
    }

    fn king_position_values() -> [[f32; 8]; 8] {
        [
            [-3.0, -4.0, -4.0, -5.0, -5.0, -4.0, -4.0, -3.0],
            [-3.0, -4.0, -4.0, -5.0, -5.0, -4.0, -4.0, -3.0],
            [-3.0, -4.0, -4.0, -5.0, -5.0, -4.0, -4.0, -3.0],
            [-3.0, -4.0, -4.0, -5.0, -5.0, -4.0, -4.0, -3.0],
            [-2.0, -3.0, -3.0, -4.0, -4.0, -3.0, -3.0, -2.0],
            [-1.0, -2.0, -2.0, -2.0, -2.0, -2.0, -2.0, -1.0],
            [ 2.0,  2.0,  0.0,  0.0,  0.0,  0.0,  2.0,  2.0],
            [ 2.0,  3.0,  1.0,  0.0,  0.0,  1.0,  3.0,  2.0],
        ]
    }
    fn find_king(&self, board: &Board, color: Color) -> Option<(usize, usize)> {
        for rank in 0..8 {
            for file in 0..8 {
                if let Some(piece) = board.get_piece((rank, file)) {
                    if piece.piece_type == PieceType::King && piece.color == color {
                        return Some((rank, file));
                    }
                }
            }
        }
        None
    }

    fn get_piece_moves(&self, board: &Board, pos: (usize, usize)) -> Vec<(usize, usize)> {
        let mut moves = Vec::new();
        for rank in 0..8 {
            for file in 0..8 {
                let to = (rank, file);
                // Create a temporary board copy to test moves
                let mut board_copy = board.clone();
                if board_copy.move_piece(pos, to) {
                    moves.push(to);
                }
            }
        }
        moves
    }

    fn evaluate_king_safety(&self, board: &Board, king_pos: (usize, usize), color: Color, analysis: &BoardAnalysis) -> f32 {
        let mut safety = 0.0;
        
        // Check surrounding squares
        for rank_offset in -1..=1 {
            for file_offset in -1..=1 {
                let rank = king_pos.0 as i32 + rank_offset;
                let file = king_pos.1 as i32 + file_offset;
                
                if rank >= 0 && rank < 8 && file >= 0 && file < 8 {
                    let pos = (rank as usize, file as usize);
                    if let Some(piece) = board.get_piece(pos) {
                        if piece.color == color {
                            safety += 1.0; // Friendly piece protecting king
                        }
                    }
                }
            }
        }
        
        // Penalize for enemy control of surrounding squares
        for &(_threat_pos, target_pos) in &analysis.threats {
            if target_pos == king_pos {
                safety -= 2.0;
            }
        }
        
        safety
    }

    fn evaluate_center_control(&self, controlled_squares: &[[bool; 8]; 8]) -> f32 {
        let mut control = 0.0;
        
        // Center squares have higher weight
        for rank in 3..=4 {
            for file in 3..=4 {
                if controlled_squares[rank][file] {
                    control += 1.0;
                }
            }
        }
        
        control
    }

    fn analyze_board(&self, board: &Board, color: Color) -> BoardAnalysis {
        let mut analysis = BoardAnalysis {
            controlled_squares: [[false; 8]; 8],
            piece_mobility: HashMap::new(),
            threats: Vec::new(),
            king_safety: 0.0,
            material_balance: 0,
            center_control: 0.0,
        };

        // Find king position
        let king_pos = self.find_king(board, color);
        
        // Analyze each square
        for rank in 0..8 {
            for file in 0..8 {
                let pos = (rank, file);
                if let Some(piece) = board.get_piece(pos) {
                    // Calculate piece mobility
                    let moves = self.get_piece_moves(board, pos);
                    // Track controlled squares
                    for &move_pos in &moves {
                        analysis.controlled_squares[move_pos.0][move_pos.1] = true;
                    }

                    // Store moves for later use
                    let moves_for_threats = moves.clone();
                    analysis.piece_mobility.insert(pos, moves.clone());

                    // Calculate material balance
                    let value = self.piece_values[&piece.piece_type];
                    if piece.color == color {
                        analysis.material_balance += value;
                    } else {
                        analysis.material_balance -= value;
                    }

                    // Identify threats
                    if piece.color != color {
                        for &target_pos in &moves_for_threats {
                            if let Some(target) = board.get_piece(target_pos) {
                                if target.color == color {
                                    analysis.threats.push((pos, target_pos));
                                }
                            }
                        }
                    }
                }
            }
        }

        // Calculate king safety
        if let Some(king_pos) = king_pos {
            analysis.king_safety = self.evaluate_king_safety(board, king_pos, color, &analysis);
        }

        // Calculate center control
        analysis.center_control = self.evaluate_center_control(&analysis.controlled_squares);

        analysis
    }

    fn is_king_threatened(&self, board: &Board, color: Color) -> bool {
        if let Some(king_pos) = self.find_king(board, color) {
            let opponent_analysis = self.analyze_board(board, color.opposite());
            opponent_analysis.controlled_squares[king_pos.0][king_pos.1]
        } else {
            false
        }
    }

    fn find_escape_move(&self, board: &Board, color: Color, analysis: &BoardAnalysis) -> Option<((usize, usize), (usize, usize))> {
        let king_pos = self.find_king(board, color)?;
        let mut best_move = None;
        let mut best_safety = f32::NEG_INFINITY;

        // Try all king moves first
        if let Some(moves) = analysis.piece_mobility.get(&king_pos) {
            for &to in moves {
                let mut board_copy = board.clone();
                if board_copy.move_piece(king_pos, to) {
                    let safety = self.evaluate_king_safety(&board_copy, to, color, analysis);
                    if safety > best_safety {
                        best_safety = safety;
                        best_move = Some((king_pos, to));
                    }
                }
            }
        }

        // If no safe king move, try blocking or capturing the threatening piece
        if best_move.is_none() {
            for &(threat_pos, target_pos) in &analysis.threats {
                for (piece_pos, moves) in &analysis.piece_mobility {
                    if *piece_pos != king_pos {
                        for &to in moves {
                            if to == threat_pos || to == target_pos {
                                let mut board_copy = board.clone();
                                if board_copy.move_piece(*piece_pos, to) {
                                    let safety = self.evaluate_king_safety(&board_copy, king_pos, color, analysis);
                                    if safety > best_safety {
                                        best_safety = safety;
                                        best_move = Some((*piece_pos, to));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        best_move
    }
}
