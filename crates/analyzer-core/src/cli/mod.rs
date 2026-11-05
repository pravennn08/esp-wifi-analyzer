//! Initial CLI grammar. Extend this without adding HAL dependencies.
mod parser;
pub use parser::parse;

use crate::model::ScanOptions;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Help,
    Status,
    Scan(ScanOptions),
}
