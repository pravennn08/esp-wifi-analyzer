//! Bounded ASCII command editor. Rejected lines cannot execute a truncated prefix.
use heapless::Vec;

use super::{Command, parse};
use crate::error::CommandError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineError {
    TooLong,
    UnsupportedCharacter,
    Receive,
    Syntax(CommandError),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputEvent {
    None,
    Echo(u8),
    Erase,
    Cancelled,
    Empty,
    Command(Command),
    Rejected(LineError),
}

#[derive(Default)]
pub struct LineEditor<const N: usize> {
    bytes: Vec<u8, N>,
    rejected: Option<LineError>,
    swallow_lf: bool,
}

impl<const N: usize> LineEditor<N> {
    pub fn line(&self) -> &[u8] {
        &self.bytes
    }

    pub fn discard(&mut self, reason: LineError) {
        self.bytes.clear();
        self.rejected = Some(reason);
        // A transport fault can lose the LF following a previously accepted CR.
        self.swallow_lf = false;
    }

    pub fn push(&mut self, byte: u8) -> InputEvent {
        if self.swallow_lf {
            self.swallow_lf = false;
            if byte == b'\n' {
                return InputEvent::None;
            }
        }

        if byte == 3 || byte == 21 {
            self.bytes.clear();
            self.rejected = None;
            return InputEvent::Cancelled;
        }

        if byte == b'\r' || byte == b'\n' {
            self.swallow_lf = byte == b'\r';
            let event = if let Some(reason) = self.rejected.take() {
                InputEvent::Rejected(reason)
            } else {
                match core::str::from_utf8(&self.bytes).map(parse) {
                    Ok(Ok(command)) => InputEvent::Command(command),
                    Ok(Err(CommandError::Empty)) => InputEvent::Empty,
                    Ok(Err(error)) => InputEvent::Rejected(LineError::Syntax(error)),
                    Err(_) => InputEvent::Rejected(LineError::UnsupportedCharacter),
                }
            };
            self.bytes.clear();
            return event;
        }

        if self.rejected.is_some() {
            return InputEvent::None;
        }

        match byte {
            8 | 127 => {
                if self.bytes.pop().is_some() {
                    InputEvent::Erase
                } else {
                    InputEvent::None
                }
            }
            b' '..=b'~' | b'\t' => {
                // Normalize tabs to spaces so terminal cursor positions stay predictable.
                let byte = if byte == b'\t' { b' ' } else { byte };
                if self.bytes.push(byte).is_ok() {
                    InputEvent::Echo(byte)
                } else {
                    self.discard(LineError::TooLong);
                    InputEvent::None
                }
            }
            _ => {
                // Never echo arbitrary terminal escape/control bytes or broken UTF-8.
                self.discard(LineError::UnsupportedCharacter);
                InputEvent::None
            }
        }
    }
}
