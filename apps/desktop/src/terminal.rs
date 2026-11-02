//! A shell running in the thread's worktree.
//!
//! The desktop talks to a real PTY: a shell starts in the checkout, a reader
//! thread feeds its output into a terminal parser, and the panel renders the
//! visible screen and forwards the keys it receives.

use std::io::{Read, Write};
use std::sync::{Arc, Mutex};

use gpui::KeyDownEvent;
use portable_pty::{Child, CommandBuilder, MasterPty, PtySize, native_pty_system};
use vt100::Parser;

/// How many rows and columns the panel opens with.
const ROWS: u16 = 24;
const COLS: u16 = 80;

/// A shell and the screen it has drawn.
pub struct Terminal {
    child: Box<dyn Child + Send>,
    /// Kept so the PTY outlives the writer and the child.
    _master: Box<dyn MasterPty + Send>,
    parser: Arc<Mutex<Parser>>,
    writer: Box<dyn Write + Send>,
}

/// One stretch of a row with the same colours.
pub struct Run {
    pub background: Option<u32>,
    /// The column the run starts at, so the renderer can place the cursor.
    pub column: u16,
    pub foreground: Option<u32>,
    pub text: String,
}

/// What the panel draws: one set of runs per row.
pub struct Snapshot {
    pub cursor: (u16, u16),
    pub rows: Vec<Vec<Run>>,
}

impl Terminal {
    /// Start the shell in `cwd`.
    pub fn spawn(cwd: &str) -> Result<Self, String> {
        let pty = native_pty_system()
            .openpty(PtySize {
                rows: ROWS,
                cols: COLS,
                pixel_width: 0,
                pixel_height: 0,
            })
            .map_err(|_| "Kloucy could not open a terminal.".to_string())?;

        let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        let mut command = CommandBuilder::new(shell);
        command.cwd(cwd);
        command.env("TERM", "xterm-256color");

        let child = pty
            .slave
            .spawn_command(command)
            .map_err(|_| "Kloucy could not start a shell.".to_string())?;
        drop(pty.slave);

        let mut reader = pty
            .master
            .try_clone_reader()
            .map_err(|_| "Kloucy could not read the terminal.".to_string())?;
        let writer = pty
            .master
            .take_writer()
            .map_err(|_| "Kloucy could not write to the terminal.".to_string())?;

        let parser = Arc::new(Mutex::new(Parser::new(ROWS, COLS, 1000)));
        let feed = parser.clone();

        std::thread::spawn(move || {
            let mut buffer = [0u8; 8192];

            while let Ok(read) = reader.read(&mut buffer) {
                if read == 0 {
                    break;
                }

                if let Ok(mut parser) = feed.lock() {
                    parser.process(&buffer[..read]);
                }
            }
        });

        Ok(Self {
            child,
            _master: pty.master,
            parser,
            writer,
        })
    }

    pub fn write(&mut self, bytes: &[u8]) {
        if bytes.is_empty() {
            return;
        }

        // Typing returns to the live screen, the way a terminal does.
        if let Ok(mut parser) = self.parser.lock() {
            parser.set_scrollback(0);
        }

        let _ = self.writer.write_all(bytes);
        let _ = self.writer.flush();
    }

    /// Move the view through the scrollback; positive is older.
    pub fn scroll_by(&mut self, lines: i32) {
        let Ok(mut parser) = self.parser.lock() else {
            return;
        };

        let current = parser.screen().scrollback();
        let next = (current as i64 + i64::from(lines)).max(0) as usize;

        parser.set_scrollback(next);
    }

    pub fn snapshot(&self) -> Snapshot {
        let Ok(parser) = self.parser.lock() else {
            return Snapshot {
                cursor: (0, 0),
                rows: Vec::new(),
            };
        };

        let screen = parser.screen();
        let (rows, cols) = screen.size();
        // The cursor is only on the live screen; history has no insertion point.
        let cursor = if screen.scrollback() == 0 {
            screen.cursor_position()
        } else {
            (u16::MAX, u16::MAX)
        };
        let mut lines = Vec::with_capacity(usize::from(rows));

        for row in 0..rows {
            let mut runs: Vec<Run> = Vec::new();

            for col in 0..cols {
                let Some(cell) = screen.cell(row, col) else {
                    continue;
                };

                let text = cell.contents();
                let foreground = rgb_of(cell.fgcolor());
                let background = rgb_of(cell.bgcolor());

                match runs.last_mut() {
                    Some(run) if run.foreground == foreground && run.background == background => {
                        run.text.push_str(&text);
                    }
                    _ => runs.push(Run {
                        background,
                        column: col,
                        foreground,
                        text,
                    }),
                }
            }

            lines.push(runs);
        }

        Snapshot {
            cursor,
            rows: lines,
        }
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.child.kill();
    }
}

/// The RGB value a terminal colour draws with, or `None` for the default.
fn rgb_of(color: vt100::Color) -> Option<u32> {
    match color {
        vt100::Color::Default => None,
        vt100::Color::Rgb(red, green, blue) => {
            Some((u32::from(red) << 16) | (u32::from(green) << 8) | u32::from(blue))
        }
        vt100::Color::Idx(index) => Some(indexed(index)),
    }
}

/// The xterm palette: sixteen named colours, a 6×6×6 cube and a grey ramp.
fn indexed(index: u8) -> u32 {
    const BASIC: [u32; 16] = [
        0x000000, 0xcd0000, 0x00cd00, 0xcdcd00, 0x0000ee, 0xcd00cd, 0x00cdcd, 0xe5e5e5, 0x7f7f7f,
        0xff0000, 0x00ff00, 0xffff00, 0x5c5cff, 0xff00ff, 0x00ffff, 0xffffff,
    ];

    match index {
        0..=15 => BASIC[usize::from(index)],
        16..=231 => {
            const LEVELS: [u32; 6] = [0, 95, 135, 175, 215, 255];
            let value = u32::from(index - 16);

            (LEVELS[(value / 36) as usize] << 16)
                | (LEVELS[((value % 36) / 6) as usize] << 8)
                | LEVELS[(value % 6) as usize]
        }
        _ => {
            let grey = 8 + 10 * u32::from(index - 232);

            (grey << 16) | (grey << 8) | grey
        }
    }
}

/// The bytes a key press sends to the shell.
pub fn bytes(event: &KeyDownEvent) -> Vec<u8> {
    let key = event.keystroke.key.as_str();
    let control = event.keystroke.modifiers.control;
    let alt = event.keystroke.modifiers.alt;
    let platform = event.keystroke.modifiers.platform;

    if control && key.len() == 1 {
        let character = key.as_bytes()[0].to_ascii_lowercase();
        let mut bytes = vec![character & 0x1f];

        if alt {
            bytes.insert(0, 0x1b);
        }

        return bytes;
    }

    let sequence: Option<Vec<u8>> = match key {
        "enter" => Some(b"\r".to_vec()),
        "tab" => Some(b"\t".to_vec()),
        "backspace" => Some(vec![0x7f]),
        "escape" => Some(vec![0x1b]),
        "up" => Some(b"\x1b[A".to_vec()),
        "down" => Some(b"\x1b[B".to_vec()),
        "right" => Some(b"\x1b[C".to_vec()),
        "left" => Some(b"\x1b[D".to_vec()),
        "home" => Some(b"\x1b[H".to_vec()),
        "end" => Some(b"\x1b[F".to_vec()),
        "delete" => Some(b"\x1b[3~".to_vec()),
        "pageup" => Some(b"\x1b[5~".to_vec()),
        "pagedown" => Some(b"\x1b[6~".to_vec()),
        _ => None,
    };

    if let Some(mut bytes) = sequence {
        if alt {
            bytes.insert(0, 0x1b);
        }

        return bytes;
    }

    if platform {
        return Vec::new();
    }

    let Some(character) = event.keystroke.key_char.as_deref() else {
        return Vec::new();
    };

    let mut bytes = Vec::with_capacity(character.len() + 1);

    if alt {
        bytes.push(0x1b);
    }

    bytes.extend_from_slice(character.as_bytes());

    bytes
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::Keystroke;

    fn key(key: &str, key_char: Option<&str>) -> KeyDownEvent {
        KeyDownEvent {
            keystroke: Keystroke {
                key: key.to_string(),
                key_char: key_char.map(str::to_string),
                modifiers: Default::default(),
            },
            is_held: false,
        }
    }

    #[test]
    fn types_a_printable_key() {
        assert_eq!(bytes(&key("a", Some("a"))), b"a");
    }

    #[test]
    fn sends_the_control_sequence_for_ctrl_c() {
        let mut event = key("c", None);
        event.keystroke.modifiers.control = true;

        assert_eq!(bytes(&event), vec![0x03]);
    }

    #[test]
    fn maps_terminal_colours_to_rgb() {
        assert_eq!(rgb_of(vt100::Color::Default), None);
        assert_eq!(rgb_of(vt100::Color::Rgb(1, 2, 3)), Some(0x010203));
        assert_eq!(rgb_of(vt100::Color::Idx(1)), Some(0xcd0000));
        assert_eq!(rgb_of(vt100::Color::Idx(16)), Some(0x000000));
        assert_eq!(rgb_of(vt100::Color::Idx(232)), Some(0x080808));
    }

    #[test]
    fn maps_enter_and_arrows() {
        assert_eq!(bytes(&key("enter", None)), b"\r");
        assert_eq!(bytes(&key("up", None)), b"\x1b[A");
        assert_eq!(bytes(&key("backspace", None)), vec![0x7f]);
    }
}
