//! Command prompt segments are the meat and potatoes of `promptr`.
//!
//! For use, see each of the respective modules
//! for available segment-specific configuration knobs.
//!
//! For development, convention dictates that
//! a new type of segment is created as a child module with three structs:
//! * A struct with a descriptive name.  It *must* implement [`ToSegment`].
//! * A struct named `Args` that contains fields that Serde can deserialize.  It *must* set the
//! `default` and *should* set the `deny_unknown_fields` Serde options.  It *must* implement the
//! `Default` trait or bad things will happen with config file processing.
//! * A struct named `Theme` that defines the themeable knobs.  The fields *should* be either
//! [`String`]s or [`Color`]s.  This struct *must* implment the [`Default`] trait.

use serde::Deserialize;

use crate::ansi::Color;
use crate::{ApplicationState, Separator};

/// Represents a rendered segment
#[derive(Debug)]
pub struct Segment {
    /// Background color
    pub bg: Color,
    /// Foreground color
    pub fg: Color,
    /// Text/emoji to be rendered
    pub text: String,
    /// Type of trailing separator to be shown. Note that the last segment always gets a "thick" separator shown after.
    pub separator: Separator,
    /// Debugging info visible from the `segment` subcommand
    pub source: &'static str,
}

/// Implement this trait for each potential segment.  It's generic over the `Deserialize` trait
/// so that each segment can have strongly typed arguments loaded from the configuration file.
pub trait ToSegment{
    type Args;
    type Theme;

    fn to_segment(args: Option<Self::Args>, state: &ApplicationState)
        -> crate::Result<Vec<Segment>>;

    /// Default impl to let us take in an untyped [`Value`](`serde_json::Value`)
    fn to_segment_generic(json: Option<serde_json::Value>, state: &ApplicationState) -> crate::Result<Vec<Segment>> where for<'de> <Self as ToSegment>::Args: Deserialize<'de> {
        let args = match json {
            Some(json) => {
                let args : Self::Args = serde_json::from_value(json)?;
                Some(args)
            },
            None => None,
        };

        Self::to_segment(args, state)
    }
}

#[cfg(feature = "segment-battery")]
pub mod battery_status;

pub mod command_status;

#[cfg(feature = "segment-git")]
pub mod git;

pub mod hostname;

pub mod paths;

#[cfg(feature = "segment-rvm")]
pub mod rvm;

pub mod username;

pub mod vcs;

#[cfg(feature = "segment-battery")]
pub use battery_status::BatteryStatus;

pub use command_status::CommandStatus;

#[cfg(feature = "segment-git")]
pub use git::Git;

pub use self::hostname::Hostname;

pub use paths::Paths;

#[cfg(feature = "segment-rvm")]
pub use rvm::Rvm;

pub use username::Username;
