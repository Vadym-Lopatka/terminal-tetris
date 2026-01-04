use std::collections::VecDeque;
use rand::Rng;

// ============================================================================
// Configuration
// ============================================================================

pub const GRID_WIDTH: usize = 10;
pub const GRID_HEIGHT: usize = 20;
pub const PREVIEW_COUNT: usize = 4;

// Timing (in milliseconds)
const BASE_TICK_MS: u64 = 800;
const MIN_TICK_MS: u64 = 100;
const SPEED_INCREASE_PER_LEVEL: u64 = 50;
pub const LINES_PER_LEVEL: u32 = 10;

// Scoring
pub const SCORE_SINGLE: u32 = 100;
pub const SCORE_DOUBLE: u32 = 300;
pub const SCORE_TRIPLE: u32 = 500;
pub const SCORE_TETRIS: u32 = 800;

// ============================================================================
// Types
// ============================================================================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Position {
    pub x: i16,
    pub y: i16,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum TetrominoType {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl TetrominoType {
    pub fn shapes(&self) -> Vec<Vec<(i16, i16)>> {
        match self {
            TetrominoType::I => vec![
                vec![(0, 0), (1, 0), (2, 0), (3, 0)],
                vec![(0, 0), (0, 1), (0, 2), (0, 3)],
                vec![(0, 0), (1, 0), (2, 0), (3, 0)],
                vec![(0, 0), (0, 1), (0, 2), (0, 3)],
            ],
            TetrominoType::O => vec![
                vec![(0, 0), (1, 0), (0, 1), (1, 1)],
                vec![(0, 0), (1, 0), (0, 1), (1, 1)],
                vec![(0, 0), (1, 0), (0, 1), (1, 1)],
                vec![(0, 0), (1, 0), (0, 1), (1, 1)],
            ],
            TetrominoType::T => vec![
                vec![(1, 0), (0, 1), (1, 1), (2, 1)],
                vec![(0, 0), (0, 1), (1, 1), (0, 2)],
                vec![(0, 0), (1, 0), (2, 0), (1, 1)],
                vec![(1, 0), (0, 1), (1, 1), (1, 2)],
            ],
            TetrominoType::S => vec![
                vec![(1, 0), (2, 0), (0, 1), (1, 1)],
                vec![(0, 0), (0, 1), (1, 1), (1, 2)],
                vec![(1, 0), (2, 0), (0, 1), (1, 1)],
                vec![(0, 0), (0, 1), (1, 1), (1, 2)],
            ],
            TetrominoType::Z => vec![
                vec![(0, 0), (1, 0), (1, 1), (2, 1)],
                vec![(1, 0), (0, 1), (1, 1), (0, 2)],
                vec![(0, 0), (1, 0), (1, 1), (2, 1)],
                vec![(1, 0), (0, 1), (1, 1), (0, 2)],
            ],
            TetrominoType::J => vec![
                vec![(0, 0), (0, 1), (1, 1), (2, 1)],
                vec![(0, 0), (1, 0), (0, 1), (0, 2)],
                vec![(0, 0), (1, 0), (2, 0), (2, 1)],
                vec![(1, 0), (1, 1), (0, 2), (1, 2)],
            ],
            TetrominoType::L => vec![
                vec![(2, 0), (0, 1), (1, 1), (2, 1)],
                vec![(0, 0), (0, 1), (0, 2), (1, 2)],
                vec![(0, 0), (1, 0), (2, 0), (0, 1)],
                vec![(0, 0), (1, 0), (1, 1), (1, 2)],
            ],
        }
    }

    fn random() -> Self {
        let mut rng = rand::thread_rng();
        match rng.gen_range(0..7) {
            0 => TetrominoType::I,
            1 => TetrominoType::O,
            2 => TetrominoType::T,
            3 => TetrominoType::S,
            4 => TetrominoType::Z,
            5 => TetrominoType::J,
            _ => TetrominoType::L,
        }
    }
}

#[derive(Clone)]
pub struct Tetromino {
    pub tetromino_type: TetrominoType,
    pub position: Position,
    pub rotation: usize,
}

impl Tetromino {
    pub fn new(tetromino_type: TetrominoType) -> Self {
        Self {
            tetromino_type,
            position: Position {
                x: (GRID_WIDTH as i16 / 2) - 1,
                y: 0,
            },
            rotation: 0,
        }
    }

    pub fn new_at(tetromino_type: TetrominoType, x: i16, y: i16) -> Self {
        Self {
            tetromino_type,
            position: Position { x, y },
            rotation: 0,
        }
    }

    pub fn blocks(&self) -> Vec<Position> {
        let shapes = self.tetromino_type.shapes();
        let shape = &shapes[self.rotation % shapes.len()];
        shape
            .iter()
            .map(|(dx, dy)| Position {
                x: self.position.x + dx,
                y: self.position.y + dy,
            })
            .collect()
    }

    fn rotated(&self, clockwise: bool) -> Self {
        let shapes = self.tetromino_type.shapes();
        let rotation = if clockwise {
            (self.rotation + 1) % shapes.len()
        } else {
            (self.rotation + shapes.len() - 1) % shapes.len()
        };
        Self {
            tetromino_type: self.tetromino_type,
            position: self.position,
            rotation,
        }
    }

    fn moved(&self, dx: i16, dy: i16) -> Self {
        Self {
            tetromino_type: self.tetromino_type,
            position: Position {
                x: self.position.x + dx,
                y: self.position.y + dy,
            },
            rotation: self.rotation,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum CellState {
    Empty,
    Filled(TetrominoType),
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum GameState {
    Playing,
    Paused,
    GameOver,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum GameEvent {
    PieceMoved,
    PieceRotated,
    PieceLocked,
    LinesCleared(u32),
    LevelUp(u32),
    Paused,
    Unpaused,
    GameRestarted,
    GameOver,
}

// ============================================================================
// Piece Provider Trait
// ============================================================================

pub trait PieceProvider {
    fn next_piece(&mut self) -> TetrominoType;
}

struct RandomPieceProvider;

impl PieceProvider for RandomPieceProvider {
    fn next_piece(&mut self) -> TetrominoType {
        TetrominoType::random()
    }
}

pub struct SequencePieceProvider {
    pieces: Vec<TetrominoType>,
    index: usize,
}

impl SequencePieceProvider {
    pub fn new(pieces: Vec<TetrominoType>) -> Self {
        Self { pieces, index: 0 }
    }
}

impl PieceProvider for SequencePieceProvider {
    fn next_piece(&mut self) -> TetrominoType {
        let piece = self.pieces[self.index % self.pieces.len()];
        self.index += 1;
        piece
    }
}

// ============================================================================
// Game
// ============================================================================

pub struct Game {
    pub grid: Vec<Vec<CellState>>,
    pub current_piece: Tetromino,
    pub preview_queue: VecDeque<TetrominoType>,
    pub score: u32,
    pub lines_cleared: u32,
    pub level: u32,
    pub high_score: u32,
    pub state: GameState,
    piece_provider: Box<dyn PieceProvider>,
    events: Vec<GameEvent>,
}

// ============================================================================
// Game Logic
// ============================================================================

const HIGH_SCORE_FILE: &str = "highscore.txt";

fn load_high_score() -> u32 {
    std::fs::read_to_string(HIGH_SCORE_FILE)
        .ok()
        .and_then(|s| s.trim().parse().ok())
        .unwrap_or(0)
}

fn save_high_score(score: u32) {
    let _ = std::fs::write(HIGH_SCORE_FILE, score.to_string());
}

impl Game {
    pub fn new() -> Self {
        Self::with_provider(Box::new(RandomPieceProvider))
    }

    pub fn with_provider(mut provider: Box<dyn PieceProvider>) -> Self {
        let grid = vec![vec![CellState::Empty; GRID_WIDTH]; GRID_HEIGHT];

        let mut preview_queue = VecDeque::new();
        for _ in 0..PREVIEW_COUNT {
            preview_queue.push_back(provider.next_piece());
        }

        let current_type = provider.next_piece();
        let current_piece = Tetromino::new(current_type);

        Self {
            grid,
            current_piece,
            preview_queue,
            score: 0,
            lines_cleared: 0,
            level: 1,
            high_score: load_high_score(),
            state: GameState::Playing,
            piece_provider: provider,
            events: Vec::new(),
        }
    }

    pub fn with_grid(grid: Vec<Vec<CellState>>, current_piece: Tetromino) -> Self {
        let mut preview_queue = VecDeque::new();
        for _ in 0..PREVIEW_COUNT {
            preview_queue.push_back(TetrominoType::random());
        }

        Self {
            grid,
            current_piece,
            preview_queue,
            score: 0,
            lines_cleared: 0,
            level: 1,
            high_score: load_high_score(),
            state: GameState::Playing,
            piece_provider: Box::new(RandomPieceProvider),
            events: Vec::new(),
        }
    }

    pub fn is_valid_position(&self, piece: &Tetromino) -> bool {
        for block in piece.blocks() {
            // Check bounds
            if block.x < 0 || block.x >= GRID_WIDTH as i16 {
                return false;
            }
            if block.y < 0 || block.y >= GRID_HEIGHT as i16 {
                return false;
            }
            // Check collision with placed blocks
            if self.grid[block.y as usize][block.x as usize] != CellState::Empty {
                return false;
            }
        }
        true
    }

    fn lock_piece(&mut self) {
        let piece_type = self.current_piece.tetromino_type;
        for block in self.current_piece.blocks() {
            if block.y >= 0 && block.y < GRID_HEIGHT as i16 {
                self.grid[block.y as usize][block.x as usize] = CellState::Filled(piece_type);
            }
        }
        self.events.push(GameEvent::PieceLocked);
    }

    pub fn clear_lines(&mut self) -> u32 {
        let mut cleared_count = 0;
        let mut y = 0;

        while y < GRID_HEIGHT {
            if self.grid[y].iter().all(|cell| *cell != CellState::Empty) {
                self.grid.remove(y);
                self.grid.insert(0, vec![CellState::Empty; GRID_WIDTH]);
                cleared_count += 1;
                // Don't increment y - the next row has shifted into this position
            } else {
                y += 1;
            }
        }

        if cleared_count > 0 {
            self.events.push(GameEvent::LinesCleared(cleared_count));
        }

        cleared_count
    }

    pub fn add_score(&mut self, lines: u32) {
        let base_score = match lines {
            1 => SCORE_SINGLE,
            2 => SCORE_DOUBLE,
            3 => SCORE_TRIPLE,
            4 => SCORE_TETRIS,
            _ => 0,
        };
        self.score += base_score * self.level;
        self.lines_cleared += lines;

        // Level up
        let new_level = (self.lines_cleared / LINES_PER_LEVEL) + 1;
        if new_level > self.level {
            self.level = new_level;
            self.events.push(GameEvent::LevelUp(self.level));
        }
    }

    pub fn spawn_next_piece(&mut self) {
        // Get next piece from queue
        let next_type = self.preview_queue.pop_front().unwrap_or_else(TetrominoType::random);
        self.preview_queue.push_back(self.piece_provider.next_piece());

        self.current_piece = Tetromino::new(next_type);

        // Check if new piece can be placed
        if !self.is_valid_position(&self.current_piece) {
            self.state = GameState::GameOver;
            self.events.push(GameEvent::GameOver);

            // Update and save high score if beaten
            if self.score > self.high_score {
                self.high_score = self.score;
                save_high_score(self.high_score);
            }
        }
    }

    pub fn move_piece(&mut self, dx: i16, dy: i16) -> bool {
        if self.state != GameState::Playing {
            return false;
        }
        let moved = self.current_piece.moved(dx, dy);
        if self.is_valid_position(&moved) {
            self.current_piece = moved;
            self.events.push(GameEvent::PieceMoved);
            true
        } else {
            false
        }
    }

    pub fn rotate_piece(&mut self, clockwise: bool) -> bool {
        if self.state != GameState::Playing {
            return false;
        }
        let rotated = self.current_piece.rotated(clockwise);
        if self.is_valid_position(&rotated) {
            self.current_piece = rotated;
            self.events.push(GameEvent::PieceRotated);
            return true;
        }

        // Wall kick attempts
        let kicks = [(1, 0), (-1, 0), (0, -1), (2, 0), (-2, 0)];
        for (dx, dy) in kicks {
            let kicked = Tetromino {
                position: Position {
                    x: rotated.position.x + dx,
                    y: rotated.position.y + dy,
                },
                ..rotated.clone()
            };
            if self.is_valid_position(&kicked) {
                self.current_piece = kicked;
                self.events.push(GameEvent::PieceRotated);
                return true;
            }
        }
        false
    }

    pub fn hard_drop(&mut self) {
        if self.state != GameState::Playing {
            return;
        }
        while self.move_piece(0, 1) {}
        // Remove the PieceMoved events from the hard drop moves (optional, but cleaner)
        self.events.retain(|e| *e != GameEvent::PieceMoved);
        self.lock_and_spawn();
    }

    pub fn soft_drop(&mut self) {
        if self.state != GameState::Playing {
            return;
        }
        if !self.move_piece(0, 1) {
            self.lock_and_spawn();
        }
    }

    fn lock_and_spawn(&mut self) {
        self.lock_piece();
        let lines = self.clear_lines();
        if lines > 0 {
            self.add_score(lines);
        }
        self.spawn_next_piece();
    }

    pub fn tick(&mut self) {
        if !matches!(self.state, GameState::Playing) {
            return;
        }

        if !self.move_piece(0, 1) {
            self.lock_and_spawn();
        }
    }

    pub fn toggle_pause(&mut self) {
        match self.state {
            GameState::Playing => {
                self.state = GameState::Paused;
                self.events.push(GameEvent::Paused);
            }
            GameState::Paused => {
                self.state = GameState::Playing;
                self.events.push(GameEvent::Unpaused);
            }
            GameState::GameOver => {
                // Cannot pause when game is over
            }
        }
    }

    pub fn restart(&mut self) {
        // Clear the grid
        self.grid = vec![vec![CellState::Empty; GRID_WIDTH]; GRID_HEIGHT];

        // Reset score, lines, and level
        self.score = 0;
        self.lines_cleared = 0;
        self.level = 1;

        // Reset state to Playing
        self.state = GameState::Playing;

        // Clear events
        self.events.clear();

        // Rebuild preview queue with new pieces
        self.preview_queue.clear();
        for _ in 0..PREVIEW_COUNT {
            self.preview_queue.push_back(self.piece_provider.next_piece());
        }

        // Spawn new current piece
        let current_type = self.piece_provider.next_piece();
        self.current_piece = Tetromino::new(current_type);

        // Emit restart event
        self.events.push(GameEvent::GameRestarted);
    }

    pub fn tick_duration_ms(&self) -> u64 {
        let speed_reduction = (self.level - 1) as u64 * SPEED_INCREASE_PER_LEVEL;
        BASE_TICK_MS.saturating_sub(speed_reduction).max(MIN_TICK_MS)
    }

    /// Returns the visual grid state with the current piece overlaid
    pub fn render_grid(&self) -> Vec<Vec<CellState>> {
        let mut visual_grid = self.grid.clone();

        // Overlay current piece
        for block in self.current_piece.blocks() {
            if block.y >= 0 && block.y < GRID_HEIGHT as i16 && block.x >= 0 && block.x < GRID_WIDTH as i16 {
                visual_grid[block.y as usize][block.x as usize] = CellState::Filled(self.current_piece.tetromino_type);
            }
        }

        visual_grid
    }

    /// Takes and clears all pending events
    pub fn take_events(&mut self) -> Vec<GameEvent> {
        std::mem::take(&mut self.events)
    }

    /// Check if a specific row is complete (all filled)
    pub fn is_row_complete(&self, y: usize) -> bool {
        self.grid[y].iter().all(|cell| *cell != CellState::Empty)
    }

    /// Count filled cells in a row
    pub fn filled_count_in_row(&self, y: usize) -> usize {
        self.grid[y].iter().filter(|cell| **cell != CellState::Empty).count()
    }

    /// Check if game is over
    pub fn is_game_over(&self) -> bool {
        self.state == GameState::GameOver
    }

    /// Count total filled cells in grid
    pub fn total_filled_cells(&self) -> usize {
        self.grid.iter().flatten().filter(|cell| **cell != CellState::Empty).count()
    }
}

impl Default for Game {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Test Helpers
// ============================================================================

pub mod test_helpers {
    use super::*;

    pub fn empty_grid() -> Vec<Vec<CellState>> {
        vec![vec![CellState::Empty; GRID_WIDTH]; GRID_HEIGHT]
    }

    pub fn fill_row(grid: &mut Vec<Vec<CellState>>, y: usize) {
        for x in 0..GRID_WIDTH {
            grid[y][x] = CellState::Filled(TetrominoType::T);
        }
    }

    pub fn fill_row_with_gap(grid: &mut Vec<Vec<CellState>>, y: usize, gap_x: usize) {
        for x in 0..GRID_WIDTH {
            if x != gap_x {
                grid[y][x] = CellState::Filled(TetrominoType::T);
            }
        }
    }
}
