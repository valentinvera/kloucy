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

/// What the panel draws: the screen as text, and where the cursor is.
pub struct Snapshot {
    pub contents: String,
    pub cursor: (u16, u16),
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

        let _ = self.writer.write_all(bytes);
        let _ = self.writer.flush();
    }

    pub fn snapshot(&self) -> Snapshot {
        let Ok(parser) = self.parser.lock() else {
            return Snapshot {
                contents: String::new(),
                cursor: (0, 0),
            };
        };

        let screen = parser.screen();

        Snapshot {
            contents: screen.contents(),
            cursor: screen.cursor_position(),
        }
    }
}

impl Drop for Terminal {
    fn drop(&mut self) {
        let _ = self.child.kill();
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
    fn maps_enter_and_arrows() {
        assert_eq!(bytes(&key("enter", None)), b"\r");
        assert_eq!(bytes(&key("up", None)), b"\x1b[A");
        assert_eq!(bytes(&key("backspace", None)), vec![0x7f]);
    }
}
