# Terminal Tetris

A classic Tetris game for the terminal, written in Rust.

![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange)
![License](https://img.shields.io/badge/license-MIT-blue)

## Install & Run

```bash
git clone https://github.com/Vadym-Lopatka/terminal-tetris.git
cd terminal-tetris
cargo run --release
```

## Controls

| Action | Keys |
|--------|------|
| Move left | `A` |
| Move right | `D` |
| Soft drop | `S` or `J` |
| Hard drop | `W` or `K` |
| Rotate ↺ | `←` or `H` |
| Rotate ↻ | `→` or `L` |
| Quit | `ESC` or `Q` |

## Configuration

Edit constants in `src/main.rs`:

```rust
const GRID_WIDTH: usize = 10;          // Board width
const GRID_HEIGHT: usize = 20;         // Board height
const PREVIEW_COUNT: usize = 4;        // Next pieces shown
const BASE_TICK_MS: u64 = 800;         // Initial speed
const MIN_TICK_MS: u64 = 100;          // Max speed
const LINES_PER_LEVEL: u32 = 10;       // Lines to level up
```

## Scoring

| Lines | Points |
|-------|--------|
| 1 | 100 × level |
| 2 | 300 × level |
| 3 | 500 × level |
| 4 | 800 × level |

## Dependencies

- [ratatui](https://github.com/ratatui-org/ratatui) — TUI framework
- [crossterm](https://github.com/crossterm-rs/crossterm) — Terminal handling
- [rand](https://github.com/rust-random/rand) — RNG
