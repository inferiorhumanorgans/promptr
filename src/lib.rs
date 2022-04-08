//! `libpromptr` is part of `promptr`, a fancy `bash` prompt generator
//!
//! This library handles all of the configuration parsing and most of the display logic.  For
//! information about what to place in your configuration files this is the place.  For usage
//! and installation information check the `promptr` documentation.

use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::fmt::{self, Display};

#[cfg(any(target_os = "macos", target_os = "freebsd"))]
#[cxx::bridge]
mod ffi {
    unsafe extern "C++" {
        include!("promptr/ffi/ffi.h");
        fn get_process_name(pid: u64) -> String;
    }
}

#[cfg(test)]
mod test;

pub mod ansi;
pub mod segment;
pub mod shell;

/// Result that takes an [`anyhow::Error`]
pub use anyhow::Result;

/// Global application state.  Includes information that we've captured from the shell and theme
/// information.
#[derive(Debug)]
pub struct ApplicationState<'a> {
    pub theme: &'a Theme,
    pub env: HashMap<String, String>,
}

/// Represents the contents of a JSON config file.
///
/// The available segments are described in the [`segment`] module.  If no config file is found,
/// the defaults are used.  Both the active and default configurations can be viewed in JSON via
/// the `promptr current-config` and `promptr default-config` commands respectively.
#[derive(Deserialize, Debug, Serialize)]
pub struct PromptrConfig {
    /// Magic number, currently needs to be 12.
    pub promptr_config: u32,

    /// List of segments to render for the left prompt.
    pub segments: Vec<SegmentConfig>,

    /// Theme options.  Each module under [`segment`] defines a Theme object with the configurable
    /// colors specific to each segment.  The only parts that need to be specified are those that
    /// you wish to override.  For instance to override only the background color for the [`Hostname`](`segment::hostname`)
    /// segment your theme stanza would look like this:
    /// ```json
    /// {
    ///     "hostname": { "bg": 128 }
    /// }
    /// ```
    ///
    /// In this case `bg` is a [`Color`](`ansi::Color`) object which can be represented by an integer.
    #[serde(default)]
    pub theme: Theme,
}

/// This represents a stanza in the config file that describes a sgement. The `args` field is typed
/// specifically for each segment, and each segment implements `serde(default)` so you only need to
/// specify the fields you wish to override.
#[derive(Deserialize, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SegmentConfig {
    pub name: String,

    #[serde(skip_serializing_if = "SegmentConfig::serialize_optional_json")]
    pub args: Option<serde_json::Value>,
}

/// Separator shown between segments
///
/// Typically the thick separator is used unless the background of two adjacent segments is the same.
#[derive(Debug)]
pub enum Separator {
    Thin,
    Thick,
}

/// Contains colors for the active theme.
///
/// All fields implement `serde(default)` and are thus optional.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Theme {
    /// Theme for the [`battery_status`](`segment::battery_status`) segment.
    #[cfg(feature = "segment-battery")]
    pub battery: segment::battery_status::Theme,

    /// Theme for the [`command_status`](`segment::command_status`) segment.
    pub command_status: segment::command_status::Theme,

    /// Theme for the [`hostname`](`segment::hostname`) segment.
    pub hostname: segment::hostname::Theme,

    /// Theme for the version control segments including the [`git`](`segment::git`) segment.
    #[cfg(feature = "segment-git")]
    pub vcs: segment::vcs::Theme,

    /// Theme for the [`username`](`segment::username`) segment.
    pub username: segment::username::Theme,

    /// Theme for the [`paths`](`segment::paths`) segment.
    pub paths: segment::paths::Theme,

    /// Theme for the [`rvm`](`segment::rvm`) segment.
    #[cfg(feature = "segment-rvm")]
    pub rvm: segment::rvm::Theme,

    /// Theme for the [`screen`](`segment::screen`) segment.
    pub screen: segment::screen::Theme,

    pub thin_separator_fg: ansi::Color,
}

impl Default for PromptrConfig {
    /// Default segments to render.
    fn default() -> Self {
        Self {
            promptr_config: 12,
            segments: vec![
                SegmentConfig {
                    name: "username".into(),
                    args: None,
                },
                SegmentConfig {
                    name: "paths".into(),
                    args: None,
                },
                SegmentConfig {
                    name: "command_status".into(),
                    args: None,
                },
            ],

            theme: Theme::default(),
        }
    }
}

impl SegmentConfig {
    /// We can end up with Some(Null) instead of None sometimes because reasons.
    /// This ensure serde skips writing those out.
    fn serialize_optional_json(value: &Option<serde_json::Value>) -> bool {
        match value {
            Some(value) => matches!(value, serde_json::Value::Null),
            None => true,
        }
    }
}

impl Display for Separator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Thin => write!(f, "\u{e0b1}"),
            Self::Thick => write!(f, "\u{e0b0}"),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            battery: Default::default(),
            command_status: Default::default(),
            hostname: Default::default(),
            vcs: Default::default(),
            username: Default::default(),
            paths: Default::default(),
            rvm: Default::default(),
            screen: Default::default(),

            thin_separator_fg: ansi::Color::Numbered(244),
        }
    }
}
