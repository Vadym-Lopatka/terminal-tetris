use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use rand::Rng;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::{
    collections::VecDeque,
    io::{self, stdout},
    time::{Duration, Instant},
};

// ============================================================================
// Configuration
// ============================================================================

const GRID_WIDTH: usize = 10;
const GRID_HEIGHT: usize = 20;
const PREVIEW_COUNT: usize = 4;

// Timing (in milliseconds)
const BASE_TICK_MS: u64 = 800;
const MIN_TICK_MS: u64 = 100;
const SPEED_INCREASE_PER_LEVEL: u64 = 50;
const LINES_PER_LEVEL: u32 = 10;

// Scoring
const SCORE_SINGLE: u32 = 100;
const SCORE_DOUBLE: u32 = 300;
const SCORE_TRIPLE: u32 = 500;
const SCORE_TETRIS: u32 = 800;

// Visual
const CELL_WIDTH: u16 = 2;
const BLOCK_CHAR: &str = "██";
const EMPTY_CHAR: &str = "  ";

// ============================================================================
// Types
// ============================================================================

#[derive(Clone, Copy, PartialEq, Eq)]
struct Position {
    x: i16,
    y: i16,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum TetrominoType {
    I,
    O,
    T,
    S,
    Z,
    J,
    L,
}

impl TetrominoType {
    fn color(&self) -> Color {
        match self {
            TetrominoType::I => Color::Cyan,
            TetrominoType::O => Color::Yellow,
            TetrominoType::T => Color::Magenta,
            TetrominoType::S => Color::Green,
            TetrominoType::Z => Color::Red,
            TetrominoType::J => Color::Blue,
            TetrominoType::L => Color::Rgb(255, 165, 0), // Orange
        }
    }

    fn shapes(&self) -> Vec<Vec<(i16, i16)>> {
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
struct Tetromino {
    tetromino_type: TetrominoType,
    position: Position,
    rotation: usize,
}

impl Tetromino {
    fn new(tetromino_type: TetrominoType) -> Self {
        Self {
            tetromino_type,
            position: Position {
                x: (GRID_WIDTH as i16 / 2) - 1,
                y: 0,
            },
            rotation: 0,
        }
    }

    fn blocks(&self) -> Vec<Position> {
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

    fn color(&self) -> Color {
        self.tetromino_type.color()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum CellState {
    Empty,
    Filled(Color),
}

enum GameState {
    Playing,
    GameOver,
}

struct Game {
    grid: Vec<Vec<CellState>>,
    current_piece: Tetromino,
    preview_queue: VecDeque<TetrominoType>,
    score: u32,
    lines_cleared: u32,
    level: u32,
    state: GameState,
}

// ============================================================================
// Game Logic
// ============================================================================

impl Game {
    fn new() -> Self {
        let grid = vec![vec![CellState::Empty; GRID_WIDTH]; GRID_HEIGHT];

        let mut preview_queue = VecDeque::new();
        for _ in 0..PREVIEW_COUNT {
            preview_queue.push_back(TetrominoType::random());
        }

        let current_type = TetrominoType::random();
        let current_piece = Tetromino::new(current_type);

        Self {
            grid,
            current_piece,
            preview_queue,
            score: 0,
            lines_cleared: 0,
            level: 1,
            state: GameState::Playing,
        }
    }

    fn is_valid_position(&self, piece: &Tetromino) -> bool {
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
        let color = self.current_piece.color();
        for block in self.current_piece.blocks() {
            if block.y >= 0 && block.y < GRID_HEIGHT as i16 {
                self.grid[block.y as usize][block.x as usize] = CellState::Filled(color);
            }
        }
    }

    fn clear_lines(&mut self) -> u32 {
        let mut lines_to_clear = Vec::new();

        for y in 0..GRID_HEIGHT {
            if self.grid[y].iter().all(|cell| *cell != CellState::Empty) {
                lines_to_clear.push(y);
            }
        }

        let cleared_count = lines_to_clear.len() as u32;

        // Remove cleared lines from bottom to top
        for &y in lines_to_clear.iter().rev() {
            self.grid.remove(y);
            self.grid.insert(0, vec![CellState::Empty; GRID_WIDTH]);
        }

        cleared_count
    }

    fn add_score(&mut self, lines: u32) {
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
        }
    }

    fn spawn_next_piece(&mut self) {
        // Get next piece from queue
        let next_type = self.preview_queue.pop_front().unwrap_or_else(TetrominoType::random);
        self.preview_queue.push_back(TetrominoType::random());

        self.current_piece = Tetromino::new(next_type);

        // Check if new piece can be placed
        if !self.is_valid_position(&self.current_piece) {
            self.state = GameState::GameOver;
        }
    }

    fn move_piece(&mut self, dx: i16, dy: i16) -> bool {
        let moved = self.current_piece.moved(dx, dy);
        if self.is_valid_position(&moved) {
            self.current_piece = moved;
            true
        } else {
            false
        }
    }

    fn rotate_piece(&mut self, clockwise: bool) {
        let rotated = self.current_piece.rotated(clockwise);
        if self.is_valid_position(&rotated) {
            self.current_piece = rotated;
            return;
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
                return;
            }
        }
    }

    fn hard_drop(&mut self) {
        while self.move_piece(0, 1) {}
        self.lock_and_spawn();
    }

    fn soft_drop(&mut self) {
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

    fn tick(&mut self) {
        if !matches!(self.state, GameState::Playing) {
            return;
        }

        if !self.move_piece(0, 1) {
            self.lock_and_spawn();
        }
    }

    fn tick_duration(&self) -> Duration {
        let speed_reduction = (self.level - 1) as u64 * SPEED_INCREASE_PER_LEVEL;
        let tick_ms = BASE_TICK_MS.saturating_sub(speed_reduction).max(MIN_TICK_MS);
        Duration::from_millis(tick_ms)
    }
}

// ============================================================================
// Rendering
// ============================================================================

fn render(frame: &mut Frame, game: &Game) {
    let area = frame.size();

    match game.state {
        GameState::Playing => render_game(frame, game, area),
        GameState::GameOver => render_game_over(frame, game, area),
    }
}

fn render_game(frame: &mut Frame, game: &Game, area: Rect) {
    // Calculate dimensions
    let grid_display_width = (GRID_WIDTH as u16 * CELL_WIDTH) + 2;
    let grid_display_height = GRID_HEIGHT as u16 + 2;
    let preview_width = 12;
    let info_width = 14;
    let total_width = grid_display_width + preview_width + info_width + 4;
    let total_height = grid_display_height + 3;

    // Center everything
    let main_area = centered_rect(total_width, total_height, area);

    // Split vertically first: game area and controls
    let vertical = Layout::vertical([
        Constraint::Length(grid_display_height),
        Constraint::Fill(1),
    ])
    .split(main_area);

    let game_row = vertical[0];

    // Layout: [Grid][Preview][Info]
    let horizontal = Layout::horizontal([
        Constraint::Length(grid_display_width),
        Constraint::Length(preview_width),
        Constraint::Length(info_width),
    ])
    .split(game_row);

    // Render game grid
    render_grid(frame, game, horizontal[0]);

    // Render preview
    render_preview(frame, game, horizontal[1]);

    // Render info panel
    render_info(frame, game, horizontal[2]);

    // Render controls hint below
    let controls_area = Rect {
        x: area.x,
        y: game_row.y + game_row.height,
        width: area.width,
        height: 2,
    };

    if controls_area.y + 1 < area.height {
        let controls = Paragraph::new(vec![
            Line::from("WASD/JK: Move/Drop | ←→/HL: Rotate | Q/ESC: Quit"),
        ])
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(controls, controls_area);
    }
}

fn render_grid(frame: &mut Frame, game: &Game, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Tetris ")
        .title_alignment(Alignment::Center);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    // Get current piece blocks for highlighting
    let current_blocks: Vec<Position> = game.current_piece.blocks();
    let current_color = game.current_piece.color();

    // Build grid display
    let mut lines: Vec<Line> = Vec::new();

    for y in 0..GRID_HEIGHT {
        let mut spans: Vec<Span> = Vec::new();

        for x in 0..GRID_WIDTH {
            let pos = Position {
                x: x as i16,
                y: y as i16,
            };

            let (symbol, style) = if current_blocks.contains(&pos) {
                (BLOCK_CHAR, Style::default().fg(current_color))
            } else {
                match game.grid[y][x] {
                    CellState::Empty => (EMPTY_CHAR, Style::default()),
                    CellState::Filled(color) => (BLOCK_CHAR, Style::default().fg(color)),
                }
            };

            spans.push(Span::styled(symbol, style));
        }

        lines.push(Line::from(spans));
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn render_preview(frame: &mut Frame, game: &Game, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Next ")
        .title_alignment(Alignment::Center);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let mut lines: Vec<Line> = Vec::new();

    for (i, &tetromino_type) in game.preview_queue.iter().take(PREVIEW_COUNT).enumerate() {
        if i > 0 {
            lines.push(Line::from(""));
        }

        let shapes = tetromino_type.shapes();
        let shape = &shapes[0];
        let color = tetromino_type.color();

        // Find bounding box
        let max_y = shape.iter().map(|(_, y)| *y).max().unwrap_or(0);

        for y in 0i16..=max_y {
            let mut spans: Vec<Span> = Vec::new();
            spans.push(Span::raw(" "));

            for x in 0i16..4i16 {
                if shape.contains(&(x, y)) {
                    spans.push(Span::styled(BLOCK_CHAR, Style::default().fg(color)));
                } else {
                    spans.push(Span::raw(EMPTY_CHAR));
                }
            }

            lines.push(Line::from(spans));
        }
    }

    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, inner);
}

fn render_info(frame: &mut Frame, game: &Game, area: Rect) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Info ")
        .title_alignment(Alignment::Center);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let lines = vec![
        Line::from(""),
        Line::from(Span::styled("Score", Style::default().fg(Color::Yellow))),
        Line::from(format!("{}", game.score)),
        Line::from(""),
        Line::from(Span::styled("Lines", Style::default().fg(Color::Cyan))),
        Line::from(format!("{}", game.lines_cleared)),
        Line::from(""),
        Line::from(Span::styled("Level", Style::default().fg(Color::Green))),
        Line::from(format!("{}", game.level)),
    ];

    let paragraph = Paragraph::new(lines).alignment(Alignment::Center);
    frame.render_widget(paragraph, inner);
}

fn render_game_over(frame: &mut Frame, game: &Game, area: Rect) {
    // First render the game in background
    render_game(frame, game, area);

    // Then overlay game over popup
    let text = vec![
        Line::from(""),
        Line::from(Span::styled(
            "GAME OVER",
            Style::default().fg(Color::Red),
        )),
        Line::from(""),
        Line::from(format!("Score: {}", game.score)),
        Line::from(format!("Lines: {}", game.lines_cleared)),
        Line::from(format!("Level: {}", game.level)),
        Line::from(""),
        Line::from(Span::styled(
            "Press ESC to quit",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Game Over ")
                .title_alignment(Alignment::Center)
                .style(Style::default().bg(Color::Black)),
        );

    let popup_area = centered_rect(24, 12, area);
    frame.render_widget(paragraph, popup_area);
}

fn centered_rect(width: u16, height: u16, area: Rect) -> Rect {
    let horizontal = Layout::horizontal([
        Constraint::Fill(1),
        Constraint::Length(width.min(area.width)),
        Constraint::Fill(1),
    ])
    .split(area);

    let vertical = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(height.min(area.height)),
        Constraint::Fill(1),
    ])
    .split(horizontal[1]);

    vertical[1]
}

// ============================================================================
// Main Loop
// ============================================================================

fn main() -> io::Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend)?;

    // Create game
    let mut game = Game::new();
    let mut last_tick = Instant::now();

    // Main loop
    loop {
        // Render
        terminal.draw(|frame| render(frame, &game))?;

        // Calculate time until next tick
        let tick_duration = game.tick_duration();
        let timeout = tick_duration
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::ZERO);

        // Handle input
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => break,
                        KeyCode::Char('a') | KeyCode::Char('A') => {
                            game.move_piece(-1, 0);
                        }
                        KeyCode::Char('d') | KeyCode::Char('D') => {
                            game.move_piece(1, 0);
                        }
                        KeyCode::Char('s') | KeyCode::Char('S') | KeyCode::Char('j') | KeyCode::Char('J') => {
                            game.soft_drop();
                        }
                        KeyCode::Char('w') | KeyCode::Char('W') | KeyCode::Char('k') | KeyCode::Char('K') => {
                            game.hard_drop();
                        }
                        KeyCode::Left | KeyCode::Char('h') | KeyCode::Char('H') => {
                            game.rotate_piece(false); // Counter-clockwise
                        }
                        KeyCode::Right | KeyCode::Char('l') | KeyCode::Char('L') => {
                            game.rotate_piece(true); // Clockwise
                        }
                        _ => {}
                    }
                }
            }
        }

        // Update game state
        if last_tick.elapsed() >= tick_duration {
            game.tick();
            last_tick = Instant::now();
        }
    }

    // Restore terminal
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    Ok(())
}
