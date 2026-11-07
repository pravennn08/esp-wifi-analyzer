// SPDX-License-Identifier: Apache-2.0

//! Initial CLI grammar. Extend this without adding HAL dependencies.
mod input;
mod parser;
pub mod report;
pub mod terminal;
pub use input::{InputEvent, LineEditor, LineError};
pub use parser::parse;

use crate::model::{Bssid, ScanOptions, WatchOptions};
use terminal::TerminalCommand;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Status,
    Scan(ScanOptions),
    Networks,
    Channels,
    History(Bssid),
    Watch(WatchOptions),
    Stop,
    Terminal(TerminalCommand),
}
