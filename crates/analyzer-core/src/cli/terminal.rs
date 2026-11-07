// SPDX-License-Identifier: Apache-2.0

//! Terminal display settings for the current console session.

pub const MIN_TERMINAL_WIDTH: usize = 40;
pub const DEFAULT_TERMINAL_WIDTH: usize = 80;
pub const MAX_TERMINAL_WIDTH: usize = 160;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalLayout {
    Auto,
    Table,
    Compact,
}

impl TerminalLayout {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Table => "table",
            Self::Compact => "compact",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TerminalSettings {
    pub color: bool,
    pub width: usize,
    pub layout: TerminalLayout,
}

impl Default for TerminalSettings {
    fn default() -> Self {
        Self {
            color: true,
            width: DEFAULT_TERMINAL_WIDTH,
            layout: TerminalLayout::Auto,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalCommand {
    Show,
    Color(bool),
    Width(usize),
    Layout(TerminalLayout),
}

#[derive(Clone, Copy)]
pub enum AnsiColor {
    Cyan,
    Green,
    Yellow,
    Red,
}

/// Style one value, resetting the terminal before any following text.
pub fn write_colored(
    out: &mut impl core::fmt::Write,
    enabled: bool,
    color: AnsiColor,
    value: impl core::fmt::Display,
) -> core::fmt::Result {
    if !enabled {
        return write!(out, "{value}");
    }
    let prefix = match color {
        AnsiColor::Cyan => "\x1b[36m",
        AnsiColor::Green => "\x1b[32m",
        AnsiColor::Yellow => "\x1b[33m",
        AnsiColor::Red => "\x1b[31m",
    };
    write!(out, "{prefix}{value}\x1b[0m")
}
