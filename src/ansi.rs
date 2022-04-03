//! Odds and ends to wrangle ANSI color live here.

use std::fmt::{self, Display};

use serde::{Deserialize, Serialize};

/// Colors that can be used with an [`AnsiCommand`]
///
/// ## Usage in a configuration file
/// To reference a color from the 256-color palette simply use an integer from 0–255. e.g.:
/// ```json
/// { "bg": 240 }
/// ```
///
/// A 24-bit color is represented by a struct like so:
/// ```json
/// {
///     "bg": {
///         "r": 255,
///         "g": 80,
///         "b": 95
///     }
/// }
/// ```
#[derive(Copy, Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Color {
    /// Color from the ANSI 256-color palette
    Numbered(u8),
    /// 24-bit "true" color
    Rgb { r: u8, g: u8, b: u8 },
}

/// ANSI SGR (Select Graphic Rendition) commands
///
/// See also: <https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_(Select_Graphic_Rendition)_parameters>
#[derive(Copy, Clone, Debug)]
#[repr(u8)]
pub enum AnsiCommand {
    // 8-bit color
    SetFgColor = 38,
    // 8-bit color
    SetBgColor = 48,
    DefaultForegroundColor = 39,
    DefaultBackgroundColor = 49,
    DefaultColorAndStyle = 0,
    BoldOn = 1,
    BoldOff = 22,
    // UnderlineOn = 4,
    // UnderlineOff = 24,
}

/// Writes an ANSI escape sequence out to a `String`
///
/// **TODO** Figure out how multi-shell support should work
pub fn escape<S: Into<Option<String>>>(cmd: AnsiCommand, args: S) -> String {
    let args = args.into();
    match args {
        Some(args) => format!(r"\[\e[{};{}m\]", cmd, args),
        None => format!(r"\[\e[{}m\]", cmd),
    }
}

impl Display for Color {
    /// Converts a [`Color`] to arguments for an ANSI command (e.g. [`AnsiCommand::SetBgColor`])
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Numbered(n) => write!(f, "5;{}", n),
            Self::Rgb { r, g, b } => write!(f, "2;{};{};{}", r, g, b),
        }
    }
}

impl Color {
    pub fn set_fg(&self) -> String {
        escape(AnsiCommand::SetFgColor, self.to_string())
    }

    pub fn set_bg(&self) -> String {
        escape(AnsiCommand::SetBgColor, self.to_string())
    }

    pub fn reset_colors() -> String {
        escape(AnsiCommand::DefaultColorAndStyle, None)
    }

    pub fn reset_bg() -> String {
        escape(AnsiCommand::DefaultBackgroundColor, None)
    }

    pub fn reset_fg() -> String {
        escape(AnsiCommand::DefaultForegroundColor, None)
    }
}

impl Display for AnsiCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", *self as u8)
    }
}
