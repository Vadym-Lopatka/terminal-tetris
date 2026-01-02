//! Comprehensive tests for Tetris game logic
//!
//! Test categories:
//! - Piece movement and collision
//! - Rotation and wall kicks
//! - Line clearing
//! - Scoring and leveling
//! - Game over detection
//! - State consistency (render_grid matches actual state)

use tetris::game::{
    test_helpers::*, CellState, Game, GameEvent, GameState, PieceProvider, Position,
    SequencePieceProvider, Tetromino, TetrominoType, GRID_HEIGHT, GRID_WIDTH, LINES_PER_LEVEL,
    SCORE_DOUBLE, SCORE_SINGLE, SCORE_TETRIS, SCORE_TRIPLE,
};

// ============================================================================
// Piece Movement Tests
// ============================================================================

mod piece_movement {
    use super::*;

    #[test]
    fn piece_moves_left() {
        let piece = Tetromino::new(TetrominoType::O);
        let mut game = Game::with_grid(empty_grid(), piece);
        let initial_x = game.current_piece.position.x;

        assert!(game.move_piece(-1, 0));
        assert_eq!(game.current_piece.position.x, initial_x - 1);
    }

    #[test]
    fn piece_moves_right() {
        let piece = Tetromino::new(TetrominoType::O);
        let mut game = Game::with_grid(empty_grid(), piece);
        let initial_x = game.current_piece.position.x;

        assert!(game.move_piece(1, 0));
        assert_eq!(game.current_piece.position.x, initial_x + 1);
    }

    #[test]
    fn piece_moves_down() {
        let piece = Tetromino::new(TetrominoType::O);
        let mut game = Game::with_grid(empty_grid(), piece);
        let initial_y = game.current_piece.position.y;

        assert!(game.move_piece(0, 1));
        assert_eq!(game.current_piece.position.y, initial_y + 1);
    }

    #[test]
    fn piece_cannot_move_through_left_wall() {
        let piece = Tetromino::new_at(TetrominoType::O, 0, 5);
        let mut game = Game::with_grid(empty_grid(), piece);

        assert!(!game.move_piece(-1, 0));
        assert_eq!(game.current_piece.position.x, 0);
    }

    #[test]
    fn piece_cannot_move_through_right_wall() {
        // O piece is 2 wide, so max x is GRID_WIDTH - 2
        let piece = Tetromino::new_at(TetrominoType::O, GRID_WIDTH as i16 - 2, 5);
        let mut game = Game::with_grid(empty_grid(), piece);

        assert!(!game.move_piece(1, 0));
        assert_eq!(game.current_piece.position.x, GRID_WIDTH as i16 - 2);
    }

    #[test]
    fn piece_cannot_move_through_floor() {
        // O piece is 2 tall, so max y is GRID_HEIGHT - 2
        let piece = Tetromino::new_at(TetrominoType::O, 4, GRID_HEIGHT as i16 - 2);
        let mut game = Game::with_grid(empty_grid(), piece);

        assert!(!game.move_piece(0, 1));
        assert_eq!(game.current_piece.position.y, GRID_HEIGHT as i16 - 2);
    }

    #[test]
    fn piece_cannot_move_into_filled_cell() {
        let mut grid = empty_grid();
        grid[10][5] = CellState::Filled(TetrominoType::O);

        let piece = Tetromino::new_at(TetrominoType::O, 4, 9);
        let mut game = Game::with_grid(grid, piece);

        // Piece occupies (4,9), (5,9), (4,10), (5,10)
        // Trying to move right would put (5,9) and (5,10) at (6,9) and (6,10)
        // But (5,10) is filled, so moving down should fail
        assert!(!game.move_piece(0, 1));
    }

    #[test]
    fn piece_emits_move_event() {
        let piece = Tetromino::new(TetrominoType::O);
        let mut game = Game::with_grid(empty_grid(), piece);
        game.take_events(); // Clear initial events

        game.move_piece(-1, 0);

        let events = game.take_events();
        assert!(events.contains(&GameEvent::PieceMoved));
    }
}

// ============================================================================
// Rotation Tests
// ============================================================================

mod rotation {
    use super::*;

    #[test]
    fn piece_rotates_clockwise() {
        let piece = Tetromino::new_at(TetrominoType::T, 4, 5);
        let mut game = Game::with_grid(empty_grid(), piece);
        let initial_rotation = game.current_piece.rotation;

        assert!(game.rotate_piece(true));
        assert_eq!(game.current_piece.rotation, (initial_rotation + 1) % 4);
    }

    #[test]
    fn piece_rotates_counter_clockwise() {
        let piece = Tetromino::new_at(TetrominoType::T, 4, 5);
        let mut game = Game::with_grid(empty_grid(), piece);

        assert!(game.rotate_piece(false));
        assert_eq!(game.current_piece.rotation, 3); // 0 - 1 wraps to 3
    }

    #[test]
    fn o_piece_rotation_is_noop() {
        let piece = Tetromino::new_at(TetrominoType::O, 4, 5);
        let mut game = Game::with_grid(empty_grid(), piece);
        let initial_blocks: Vec<Position> = game.current_piece.blocks();

        game.rotate_piece(true);
        let after_blocks: Vec<Position> = game.current_piece.blocks();

        // O piece looks the same after rotation
        assert_eq!(initial_blocks, after_blocks);
    }

    #[test]
    fn wall_kick_right() {
        // Place T piece against left wall, rotation should kick it right
        let piece = Tetromino::new_at(TetrominoType::T, 0, 5);
        let mut game = Game::with_grid(empty_grid(), piece);

        // This rotation might need a wall kick
        assert!(game.rotate_piece(true));
    }

    #[test]
    fn rotation_emits_event() {
        let piece = Tetromino::new_at(TetrominoType::T, 4, 5);
        let mut game = Game::with_grid(empty_grid(), piece);
        game.take_events();

        game.rotate_piece(true);

        let events = game.take_events();
        assert!(events.contains(&GameEvent::PieceRotated));
    }
}

// ============================================================================
// Line Clearing Tests
// ============================================================================

mod line_clearing {
    use super::*;

    #[test]
    fn single_complete_row_is_cleared() {
        let mut grid = empty_grid();
        fill_row(&mut grid, GRID_HEIGHT - 1);

        // Use I piece horizontally at top (won't interfere)
        let piece = Tetromino::new_at(TetrominoType::I, 0, 0);
        let mut game = Game::with_grid(grid, piece);

        // Verify row is complete
        assert!(game.is_row_complete(GRID_HEIGHT - 1));

        let cleared = game.clear_lines();

        assert_eq!(cleared, 1);
        assert!(!game.is_row_complete(GRID_HEIGHT - 1));
        assert_eq!(game.filled_count_in_row(GRID_HEIGHT - 1), 0);
    }

    #[test]
    fn multiple_rows_cleared_simultaneously() {
        let mut grid = empty_grid();
        fill_row(&mut grid, GRID_HEIGHT - 1);
        fill_row(&mut grid, GRID_HEIGHT - 2);

        let piece = Tetromino::new_at(TetrominoType::I, 0, 0);
        let mut game = Game::with_grid(grid, piece);

        let cleared = game.clear_lines();

        assert_eq!(cleared, 2);
        // After clearing 2 bottom rows, the bottom rows should now be empty
        // (the filled rows were removed and empty rows inserted at top)
        assert!(!game.is_row_complete(GRID_HEIGHT - 1));
        assert!(!game.is_row_complete(GRID_HEIGHT - 2));
    }

    #[test]
    fn tetris_clears_four_rows() {
        let mut grid = empty_grid();
        for i in 0..4 {
            fill_row(&mut grid, GRID_HEIGHT - 1 - i);
        }

        let piece = Tetromino::new_at(TetrominoType::I, 0, 0);
        let mut game = Game::with_grid(grid, piece);

        let cleared = game.clear_lines();

        assert_eq!(cleared, 4);
    }

    #[test]
    fn incomplete_row_not_cleared() {
        let mut grid = empty_grid();
        fill_row_with_gap(&mut grid, GRID_HEIGHT - 1, 5);

        let piece = Tetromino::new_at(TetrominoType::I, 0, 0);
        let mut game = Game::with_grid(grid, piece);

        assert!(!game.is_row_complete(GRID_HEIGHT - 1));

        let cleared = game.clear_lines();

        assert_eq!(cleared, 0);
        assert_eq!(game.filled_count_in_row(GRID_HEIGHT - 1), GRID_WIDTH - 1);
    }

    #[test]
    fn rows_above_cleared_line_fall_down() {
        let mut grid = empty_grid();
        // Fill bottom row completely
        fill_row(&mut grid, GRID_HEIGHT - 1);
        // Put some blocks in the row above
        grid[GRID_HEIGHT - 2][0] = CellState::Filled(TetrominoType::T);
        grid[GRID_HEIGHT - 2][1] = CellState::Filled(TetrominoType::T);

        let piece = Tetromino::new_at(TetrominoType::I, 5, 0);
        let mut game = Game::with_grid(grid, piece);

        game.clear_lines();

        // The blocks from row GRID_HEIGHT-2 should now be at GRID_HEIGHT-1
        assert_eq!(
            game.grid[GRID_HEIGHT - 1][0],
            CellState::Filled(TetrominoType::T)
        );
        assert_eq!(
            game.grid[GRID_HEIGHT - 1][1],
            CellState::Filled(TetrominoType::T)
        );
    }

    #[test]
    fn clear_lines_emits_event() {
        let mut grid = empty_grid();
        fill_row(&mut grid, GRID_HEIGHT - 1);

        let piece = Tetromino::new_at(TetrominoType::I, 0, 0);
        let mut game = Game::with_grid(grid, piece);
        game.take_events();

        game.clear_lines();

        let events = game.take_events();
        assert!(events.contains(&GameEvent::LinesCleared(1)));
    }

    #[test]
    fn non_contiguous_rows_cleared() {
        let mut grid = empty_grid();
        fill_row(&mut grid, GRID_HEIGHT - 1); // Bottom row
        fill_row(&mut grid, GRID_HEIGHT - 3); // Skip one row

        let piece = Tetromino::new_at(TetrominoType::I, 0, 0);
        let mut game = Game::with_grid(grid, piece);

        let cleared = game.clear_lines();

        assert_eq!(cleared, 2);
    }
}

// ============================================================================
// Scoring Tests
// ============================================================================

mod scoring {
    use super::*;

    #[test]
    fn single_line_scores_correctly() {
        let piece = Tetromino::new(TetrominoType::O);
        let mut game = Game::with_grid(empty_grid(), piece);

        game.add_score(1);

        assert_eq!(game.score, SCORE_SINGLE);
        assert_eq!(game.lines_cleared, 1);
    }

    #[test]
    fn double_line_scores_correctly() {
        let piece = Tetromino::new(TetrominoType::O);
        let mut game = Game::with_grid(empty_grid(), piece);

        game.add_score(2);

        assert_eq!(game.score, SCORE_DOUBLE);
    }

    #[test]
    fn triple_line_scores_correctly() {
        let piece = Tetromino::new(TetrominoType::O);
        let mut game = Game::with_grid(empty_grid(), piece);

        game.add_score(3);

        assert_eq!(game.score, SCORE_TRIPLE);
    }

    #[test]
    fn tetris_scores_correctly() {
        let piece = Tetromino::new(TetrominoType::O);
        let mut game = Game::with_grid(empty_grid(), piece);

        game.add_score(4);

        assert_eq!(game.score, SCORE_TETRIS);
    }

    #[test]
    fn score_multiplied_by_level() {
        let piece = Tetromino::new(TetrominoType::O);
        let mut game = Game::with_grid(empty_grid(), piece);
        game.level = 3;

        game.add_score(1);

        assert_eq!(game.score, SCORE_SINGLE * 3);
    }

    #[test]
    fn level_increases_after_lines_threshold() {
        let piece = Tetromino::new(TetrominoType::O);
        let mut game = Game::with_grid(empty_grid(), piece);

        assert_eq!(game.level, 1);

        // Clear enough lines to level up
        game.add_score(LINES_PER_LEVEL);

        assert_eq!(game.level, 2);
    }

    #[test]
    fn level_up_emits_event() {
        let piece = Tetromino::new(TetrominoType::O);
        let mut game = Game::with_grid(empty_grid(), piece);
        game.take_events();

        game.add_score(LINES_PER_LEVEL);

        let events = game.take_events();
        assert!(events.contains(&GameEvent::LevelUp(2)));
    }
}

// ============================================================================
// Hard Drop Tests
// ============================================================================

mod hard_drop {
    use super::*;

    #[test]
    fn hard_drop_moves_piece_to_bottom() {
        let piece = Tetromino::new_at(TetrominoType::O, 4, 0);
        let mut game = Game::with_grid(empty_grid(), piece);

        game.hard_drop();

        // O piece should be locked at bottom (y = GRID_HEIGHT - 2)
        // Check that cells are filled
        assert_ne!(
            game.grid[GRID_HEIGHT - 1][4],
            CellState::Empty
        );
        assert_ne!(
            game.grid[GRID_HEIGHT - 1][5],
            CellState::Empty
        );
    }

    #[test]
    fn hard_drop_locks_piece_immediately() {
        let piece = Tetromino::new_at(TetrominoType::O, 4, 0);
        let mut game = Game::with_grid(empty_grid(), piece);
        game.take_events();

        game.hard_drop();

        let events = game.take_events();
        assert!(events.contains(&GameEvent::PieceLocked));
    }

    #[test]
    fn hard_drop_spawns_new_piece() {
        let pieces = vec![TetrominoType::O, TetrominoType::T, TetrominoType::I];
        let provider = Box::new(SequencePieceProvider::new(pieces));
        let mut game = Game::with_provider(provider);

        let first_piece_type = game.current_piece.tetromino_type;
        game.hard_drop();

        assert_ne!(game.current_piece.tetromino_type, first_piece_type);
    }

    #[test]
    fn hard_drop_clears_lines() {
        let mut grid = empty_grid();
        // Fill bottom row except for columns 4 and 5 (where O piece will land)
        for x in 0..GRID_WIDTH {
            if x != 4 && x != 5 {
                grid[GRID_HEIGHT - 1][x] = CellState::Filled(TetrominoType::T);
                grid[GRID_HEIGHT - 2][x] = CellState::Filled(TetrominoType::T);
            }
        }

        let piece = Tetromino::new_at(TetrominoType::O, 4, 0);
        let mut game = Game::with_grid(grid, piece);
        game.take_events();

        game.hard_drop();

        let events = game.take_events();
        assert!(events.iter().any(|e| matches!(e, GameEvent::LinesCleared(2))));
    }
}

// ============================================================================
// Soft Drop Tests
// ============================================================================

mod soft_drop {
    use super::*;

    #[test]
    fn soft_drop_moves_piece_down_one() {
        let piece = Tetromino::new_at(TetrominoType::O, 4, 0);
        let mut game = Game::with_grid(empty_grid(), piece);

        game.soft_drop();

        assert_eq!(game.current_piece.position.y, 1);
    }

    #[test]
    fn soft_drop_locks_when_at_bottom() {
        let piece = Tetromino::new_at(TetrominoType::O, 4, GRID_HEIGHT as i16 - 2);
        let mut game = Game::with_grid(empty_grid(), piece);
        game.take_events();

        game.soft_drop();

        let events = game.take_events();
        assert!(events.contains(&GameEvent::PieceLocked));
    }

    #[test]
    fn soft_drop_locks_when_blocked() {
        let mut grid = empty_grid();
        grid[GRID_HEIGHT - 1][4] = CellState::Filled(TetrominoType::T);

        let piece = Tetromino::new_at(TetrominoType::O, 4, GRID_HEIGHT as i16 - 3);
        let mut game = Game::with_grid(grid, piece);
        game.take_events();

        game.soft_drop();

        let events = game.take_events();
        assert!(events.contains(&GameEvent::PieceLocked));
    }
}

// ============================================================================
// Game Over Tests
// ============================================================================

mod game_over {
    use super::*;

    #[test]
    fn game_over_when_spawn_blocked() {
        let mut grid = empty_grid();
        // Fill the spawn area
        for x in 3..7 {
            grid[0][x] = CellState::Filled(TetrominoType::T);
            grid[1][x] = CellState::Filled(TetrominoType::T);
        }

        let piece = Tetromino::new_at(TetrominoType::O, 0, 10); // Current piece away from spawn
        let mut game = Game::with_grid(grid, piece);

        game.spawn_next_piece();

        assert!(game.is_game_over());
    }

    #[test]
    fn game_over_emits_event() {
        let mut grid = empty_grid();
        for x in 0..GRID_WIDTH {
            grid[0][x] = CellState::Filled(TetrominoType::T);
            grid[1][x] = CellState::Filled(TetrominoType::T);
        }

        let piece = Tetromino::new_at(TetrominoType::O, 0, 10);
        let mut game = Game::with_grid(grid, piece);
        game.take_events();

        game.spawn_next_piece();

        let events = game.take_events();
        assert!(events.contains(&GameEvent::GameOver));
    }

    #[test]
    fn no_moves_after_game_over() {
        let piece = Tetromino::new(TetrominoType::O);
        let mut game = Game::with_grid(empty_grid(), piece);
        game.state = GameState::GameOver;

        assert!(!game.move_piece(-1, 0));
        assert!(!game.rotate_piece(true));
    }
}

// ============================================================================
// Render Grid Consistency Tests
// ============================================================================

mod render_consistency {
    use super::*;

    #[test]
    fn render_grid_includes_current_piece() {
        let piece = Tetromino::new_at(TetrominoType::O, 4, 5);
        let game = Game::with_grid(empty_grid(), piece);

        let visual = game.render_grid();

        // O piece at (4,5) occupies (4,5), (5,5), (4,6), (5,6)
        assert_eq!(visual[5][4], CellState::Filled(TetrominoType::O));
        assert_eq!(visual[5][5], CellState::Filled(TetrominoType::O));
        assert_eq!(visual[6][4], CellState::Filled(TetrominoType::O));
        assert_eq!(visual[6][5], CellState::Filled(TetrominoType::O));
    }

    #[test]
    fn render_grid_includes_locked_pieces() {
        let mut grid = empty_grid();
        grid[GRID_HEIGHT - 1][0] = CellState::Filled(TetrominoType::T);

        let piece = Tetromino::new_at(TetrominoType::O, 4, 0);
        let game = Game::with_grid(grid, piece);

        let visual = game.render_grid();

        assert_eq!(visual[GRID_HEIGHT - 1][0], CellState::Filled(TetrominoType::T));
    }

    #[test]
    fn render_grid_matches_after_line_clear() {
        let mut grid = empty_grid();
        fill_row(&mut grid, GRID_HEIGHT - 1);
        // Add a marker block above
        grid[GRID_HEIGHT - 2][0] = CellState::Filled(TetrominoType::J);

        let piece = Tetromino::new_at(TetrominoType::O, 4, 0);
        let mut game = Game::with_grid(grid, piece);

        game.clear_lines();
        let visual = game.render_grid();

        // After clearing, the J block should have fallen to bottom row
        assert_eq!(visual[GRID_HEIGHT - 1][0], CellState::Filled(TetrominoType::J));
        // And the cleared row cells should be empty (except current piece if overlapping)
        // Check a cell not covered by current piece
        assert_eq!(visual[GRID_HEIGHT - 1][9], CellState::Empty);
    }

    #[test]
    fn render_grid_current_piece_overlays_correctly() {
        // Edge case: what if current piece position overlaps with grid cell visually?
        // render_grid should show the current piece
        let mut grid = empty_grid();
        grid[5][4] = CellState::Filled(TetrominoType::T); // Place a T block

        let piece = Tetromino::new_at(TetrominoType::O, 4, 5); // O piece overlaps at (4,5)
        let game = Game::with_grid(grid, piece);

        let visual = game.render_grid();

        // Current piece should be shown (O), not the underlying T
        assert_eq!(visual[5][4], CellState::Filled(TetrominoType::O));
    }
}

// ============================================================================
// Deterministic Piece Provider Tests
// ============================================================================

mod piece_provider {
    use super::*;

    #[test]
    fn sequence_provider_cycles() {
        let mut provider = SequencePieceProvider::new(vec![TetrominoType::I, TetrominoType::O]);

        assert_eq!(provider.next_piece(), TetrominoType::I);
        assert_eq!(provider.next_piece(), TetrominoType::O);
        assert_eq!(provider.next_piece(), TetrominoType::I); // Cycles
    }

    #[test]
    fn game_uses_provider_for_pieces() {
        let pieces = vec![
            TetrominoType::T,  // preview[0]
            TetrominoType::S,  // preview[1]
            TetrominoType::Z,  // preview[2]
            TetrominoType::L,  // preview[3]
            TetrominoType::J,  // current piece (5th drawn)
            TetrominoType::I,  // will be added to preview after spawn
        ];
        let provider = Box::new(SequencePieceProvider::new(pieces.clone()));
        let game = Game::with_provider(provider);

        // Current piece is the 5th one drawn (after 4 go to preview)
        assert_eq!(game.current_piece.tetromino_type, TetrominoType::J);

        // Preview queue should have first 4 pieces
        let preview: Vec<_> = game.preview_queue.iter().copied().collect();
        assert_eq!(preview[0], TetrominoType::T);
        assert_eq!(preview[1], TetrominoType::S);
        assert_eq!(preview[2], TetrominoType::Z);
        assert_eq!(preview[3], TetrominoType::L);
    }
}

// ============================================================================
// Tick Tests
// ============================================================================

mod tick {
    use super::*;

    #[test]
    fn tick_moves_piece_down() {
        let piece = Tetromino::new_at(TetrominoType::O, 4, 0);
        let mut game = Game::with_grid(empty_grid(), piece);

        game.tick();

        assert_eq!(game.current_piece.position.y, 1);
    }

    #[test]
    fn tick_locks_piece_at_bottom() {
        let piece = Tetromino::new_at(TetrominoType::O, 4, GRID_HEIGHT as i16 - 2);
        let mut game = Game::with_grid(empty_grid(), piece);
        game.take_events();

        game.tick();

        let events = game.take_events();
        assert!(events.contains(&GameEvent::PieceLocked));
    }

    #[test]
    fn tick_does_nothing_when_game_over() {
        let piece = Tetromino::new_at(TetrominoType::O, 4, 5);
        let mut game = Game::with_grid(empty_grid(), piece);
        game.state = GameState::GameOver;
        let initial_y = game.current_piece.position.y;

        game.tick();

        assert_eq!(game.current_piece.position.y, initial_y);
    }
}

// ============================================================================
// Integration Tests - Full Game Scenarios
// ============================================================================

mod integration {
    use super::*;

    #[test]
    fn complete_game_scenario_line_clear() {
        // Setup: Almost complete bottom row, drop I piece to complete it
        let mut grid = empty_grid();
        for x in 0..6 {
            grid[GRID_HEIGHT - 1][x] = CellState::Filled(TetrominoType::T);
        }

        // I piece horizontal at position that will fill columns 6-9
        let pieces = vec![TetrominoType::I, TetrominoType::O];
        let provider = Box::new(SequencePieceProvider::new(pieces));
        let mut game = Game::with_provider(provider);
        game.grid = grid;
        game.current_piece = Tetromino::new_at(TetrominoType::I, 6, 0);
        game.take_events();

        // Hard drop the I piece
        game.hard_drop();

        // Should have cleared the line
        let events = game.take_events();
        assert!(events.iter().any(|e| matches!(e, GameEvent::LinesCleared(1))));
        assert_eq!(game.lines_cleared, 1);
        assert_eq!(game.score, SCORE_SINGLE);
    }

    #[test]
    fn complete_game_scenario_tetris() {
        // Setup: 4 almost complete rows
        let mut grid = empty_grid();
        for y in (GRID_HEIGHT - 4)..GRID_HEIGHT {
            for x in 0..9 {
                grid[y][x] = CellState::Filled(TetrominoType::T);
            }
        }

        let pieces = vec![TetrominoType::I, TetrominoType::O];
        let provider = Box::new(SequencePieceProvider::new(pieces));
        let mut game = Game::with_provider(provider);
        game.grid = grid;
        // I piece vertical at column 9
        game.current_piece = Tetromino::new_at(TetrominoType::I, 9, 0);
        game.current_piece.rotation = 1; // Vertical
        game.take_events();

        game.hard_drop();

        let events = game.take_events();
        assert!(events.iter().any(|e| matches!(e, GameEvent::LinesCleared(4))));
        assert_eq!(game.score, SCORE_TETRIS);
    }

    #[test]
    fn rapid_soft_drops_work_correctly() {
        let piece = Tetromino::new_at(TetrominoType::O, 4, 0);
        let mut game = Game::with_grid(empty_grid(), piece);

        // Soft drop all the way down
        for _ in 0..30 {
            game.soft_drop();
            if game.current_piece.position.y == 0 {
                // New piece spawned
                break;
            }
        }

        // Should have locked and spawned new piece
        assert!(game.total_filled_cells() > 0);
    }

    #[test]
    fn game_state_consistent_after_many_operations() {
        let pieces: Vec<TetrominoType> = vec![
            TetrominoType::T,
            TetrominoType::S,
            TetrominoType::Z,
            TetrominoType::L,
            TetrominoType::J,
            TetrominoType::I,
            TetrominoType::O,
        ];
        let provider = Box::new(SequencePieceProvider::new(pieces));
        let mut game = Game::with_provider(provider);

        // Simulate some gameplay
        for _ in 0..10 {
            game.move_piece(-1, 0);
            game.move_piece(1, 0);
            game.rotate_piece(true);
            game.hard_drop();

            if game.is_game_over() {
                break;
            }
        }

        // Verify render_grid is valid
        let visual = game.render_grid();
        assert_eq!(visual.len(), GRID_HEIGHT);
        assert_eq!(visual[0].len(), GRID_WIDTH);

        // Every cell should be a valid CellState
        for row in &visual {
            for cell in row {
                match cell {
                    CellState::Empty => {}
                    CellState::Filled(_) => {}
                }
            }
        }
    }
}

// ============================================================================
// Edge Cases
// ============================================================================

mod edge_cases {
    use super::*;

    #[test]
    fn i_piece_at_top_row() {
        // I piece horizontal at y=0 should be valid
        let piece = Tetromino::new_at(TetrominoType::I, 0, 0);
        let game = Game::with_grid(empty_grid(), piece);

        assert!(game.is_valid_position(&game.current_piece));
    }

    #[test]
    fn piece_at_exact_boundaries() {
        // Test pieces at exact grid boundaries
        let test_cases = vec![
            (TetrominoType::O, 0, 0),                                         // Top-left
            (TetrominoType::O, GRID_WIDTH as i16 - 2, 0),                     // Top-right
            (TetrominoType::O, 0, GRID_HEIGHT as i16 - 2),                    // Bottom-left
            (TetrominoType::O, GRID_WIDTH as i16 - 2, GRID_HEIGHT as i16 - 2), // Bottom-right
        ];

        for (piece_type, x, y) in test_cases {
            let piece = Tetromino::new_at(piece_type, x, y);
            let game = Game::with_grid(empty_grid(), piece);
            assert!(
                game.is_valid_position(&game.current_piece),
                "Piece at ({}, {}) should be valid",
                x,
                y
            );
        }
    }

    #[test]
    fn clear_top_row() {
        let mut grid = empty_grid();
        fill_row(&mut grid, 0); // Fill top row

        let piece = Tetromino::new_at(TetrominoType::O, 4, 10);
        let mut game = Game::with_grid(grid, piece);

        let cleared = game.clear_lines();

        assert_eq!(cleared, 1);
        assert_eq!(game.filled_count_in_row(0), 0);
    }

    #[test]
    fn all_rows_filled_and_cleared() {
        let mut grid = empty_grid();
        for y in 0..GRID_HEIGHT {
            fill_row(&mut grid, y);
        }

        let piece = Tetromino::new_at(TetrominoType::O, 4, 0);
        let mut game = Game::with_grid(grid, piece);

        let cleared = game.clear_lines();

        assert_eq!(cleared, GRID_HEIGHT as u32);
        // After clearing all rows, grid should be empty
        for y in 0..GRID_HEIGHT {
            assert!(!game.is_row_complete(y));
        }
    }
}
