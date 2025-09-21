use crossterm::event::KeyCode;
use ratatui::style::Color;

// === general ===
pub const APP_VER: &str = env!("CARGO_PKG_VERSION");
pub const HIL_CLR: Color = Color::Green;
pub const COPYRIGHT: &str = "© 2025 stim — MIT License";
pub const WEBSITE: &str = "www.simondanielsson.se";

// === keymaps ===

pub const K_QUIT: KeyCode = KeyCode::Char('q'); // quit
pub const K_FIND: KeyCode = KeyCode::Char('f'); // find
pub const K_CLEAR_FIND: KeyCode = KeyCode::Char('F'); // find

pub const K_PLAY: KeyCode = KeyCode::Char(' '); // play/pause

pub const K_VOL_UP: KeyCode = KeyCode::Char(']'); // volume up
pub const K_VOL_DOWN: KeyCode = KeyCode::Char('['); // volume down

pub const K_N_TRK: KeyCode = KeyCode::Char('}'); // next track
pub const K_P_TRK: KeyCode = KeyCode::Char('{'); // previous track

pub const K_LEFT: &[KeyCode] = &[KeyCode::Left, KeyCode::Char('n'), KeyCode::Char('h')]; // move left
pub const K_RIGHT: &[KeyCode] = &[KeyCode::Right, KeyCode::Char('i'), KeyCode::Char('l')]; // move right
pub const K_DOWN: &[KeyCode] = &[KeyCode::Down, KeyCode::Char('e'), KeyCode::Char('j')]; // move down
pub const K_UP: &[KeyCode] = &[KeyCode::Up, KeyCode::Char('o'), KeyCode::Char('k')]; // move up

pub const K_CLEAR: KeyCode = KeyCode::Esc; // clear queue
pub const K_MAIN: KeyCode = KeyCode::Char('a'); // main action
pub const K_AUX: KeyCode = KeyCode::Char('A'); // aux action
