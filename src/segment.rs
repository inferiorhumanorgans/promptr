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

use crate::{ApplicationState, Color, Separator};

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
    /// Debugging info visible from the `dump-segment` subcommand
    pub source: &'static str,
}

/// Implement this trait for each potential segment.  It's generic over the `Deserialize` trait
/// so that each segment can have strongly typed arguments loaded from the configuration file.
pub trait ToSegment
{
    type Args: Deserialize<'static>;
    type Theme: Deserialize<'static>;

    fn to_segment(arg: Option<Self::Args>, state: &ApplicationState)
        -> crate::Result<Vec<Segment>>;
}

#[cfg(feature = "segment-battery")]
pub mod battery_status;

pub mod command_status;

#[cfg(feature = "segment-git")]
pub mod git;

pub mod hostname;

pub mod paths;

pub mod username;

pub mod vcs;

#[cfg(feature = "segment-battery")]
pub(super) use battery_status::BatteryStatus;

pub(super) use command_status::CommandStatus;

#[cfg(feature = "segment-git")]
pub(super) use git::Git;

pub(super) use self::hostname::Hostname;

pub(super) use paths::Paths;

pub(super) use username::Username;
