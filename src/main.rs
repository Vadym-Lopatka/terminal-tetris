use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph},
    Frame, Terminal,
};
use std::{
    io::{self, stdout},
    time::{Duration, Instant},
};

use tetris::game::{CellState, Game, GameState, TetrominoType, GRID_HEIGHT, GRID_WIDTH, PREVIEW_COUNT};

// ============================================================================
// Visual Constants
// ============================================================================

const CELL_WIDTH: u16 = 2;
const BLOCK_CHAR: &str = "██";
const EMPTY_CHAR: &str = "  ";

// ============================================================================
// Color Mapping
// ============================================================================

fn tetromino_color(t: TetrominoType) -> Color {
    match t {
        TetrominoType::I => Color::Cyan,
        TetrominoType::O => Color::Yellow,
        TetrominoType::T => Color::Magenta,
        TetrominoType::S => Color::Green,
        TetrominoType::Z => Color::Red,
        TetrominoType::J => Color::Blue,
        TetrominoType::L => Color::Rgb(255, 165, 0),
    }
}

// ============================================================================
// Rendering
// ============================================================================

fn render(frame: &mut Frame, game: &Game) {
    let area = frame.size();

    match game.state {
        GameState::Playing => render_game(frame, game, area),
        GameState::Paused => render_paused(frame, game, area),
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
        let controls = Paragraph::new(vec![Line::from(
            "WASD/JK: Move/Drop | ←→/HL: Rotate | P: Pause | Q/ESC: Quit",
        )])
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

    // Get the complete visual grid state from game logic
    // This ensures rendering always matches game state
    let visual_grid = game.render_grid();

    // Build grid display
    let mut lines: Vec<Line> = Vec::new();

    for y in 0..GRID_HEIGHT {
        let mut spans: Vec<Span> = Vec::new();

        for x in 0..GRID_WIDTH {
            let (symbol, style) = match visual_grid[y][x] {
                CellState::Empty => (EMPTY_CHAR, Style::default()),
                CellState::Filled(piece_type) => {
                    (BLOCK_CHAR, Style::default().fg(tetromino_color(piece_type)))
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
        let color = tetromino_color(tetromino_type);

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
        Line::from(Span::styled("GAME OVER", Style::default().fg(Color::Red))),
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

    let paragraph = Paragraph::new(text).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Game Over ")
            .title_alignment(Alignment::Center)
            .style(Style::default().bg(Color::Black)),
    );

    let popup_area = centered_rect(24, 12, area);
    frame.render_widget(paragraph, popup_area);
}

fn render_paused(frame: &mut Frame, game: &Game, area: Rect) {
    // First render the game in background
    render_game(frame, game, area);

    // Then overlay paused popup
    let text = vec![
        Line::from(""),
        Line::from(Span::styled("PAUSED", Style::default().fg(Color::Yellow))),
        Line::from(""),
        Line::from(Span::styled(
            "Press P to continue",
            Style::default().fg(Color::DarkGray),
        )),
        Line::from(Span::styled(
            "Press ESC to quit",
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(text).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .title(" Paused ")
            .title_alignment(Alignment::Center)
            .style(Style::default().bg(Color::Black)),
    );

    let popup_area = centered_rect(24, 10, area);
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
        let tick_duration = Duration::from_millis(game.tick_duration_ms());
        let timeout = tick_duration
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::ZERO);

        // Handle input
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        // Always allow quit
                        KeyCode::Esc | KeyCode::Char('q') | KeyCode::Char('Q') => break,
                        // Always allow pause/unpause toggle
                        KeyCode::Char('p') | KeyCode::Char('P') => {
                            game.toggle_pause();
                        }
                        // Only process game controls when playing
                        _ if game.state == GameState::Playing => {
                            match key.code {
                                KeyCode::Char('a') | KeyCode::Char('A') => {
                                    game.move_piece(-1, 0);
                                }
                                KeyCode::Char('d') | KeyCode::Char('D') => {
                                    game.move_piece(1, 0);
                                }
                                KeyCode::Char('s') | KeyCode::Char('S')
                                | KeyCode::Char('j') | KeyCode::Char('J') => {
                                    game.soft_drop();
                                }
                                KeyCode::Char('w') | KeyCode::Char('W')
                                | KeyCode::Char('k') | KeyCode::Char('K') => {
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
