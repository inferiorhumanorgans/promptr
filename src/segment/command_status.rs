//! The `CommandStatus` segment displays the root / non-privileged indicator and the last exit value

use serde::{Deserialize, Serialize};

use crate::ansi::Color;
use crate::segment::{Segment, ToSegment};
use crate::{ApplicationState, Separator};

pub struct CommandStatus {}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Args {}

/// Theme for the [`CommandStatus`] segment.
///
/// TODO: Make the exit status coloring optional
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Theme {
    /// Foreground color when the exit status is zero
    pub success_fg: Color,
    /// Background color when the exit status is zero
    pub success_bg: Color,

    /// Foreground color when the exit status is non-zero
    pub failure_fg: Color,
    /// Background color when the exit status is non-zero
    pub failure_bg: Color,

    /// Indicator used when user is super.  On bash this is typically `#`.
    pub root_indicator: String,

    /// Indicator for non-privileged users.  On bash this is typically `$`.
    pub user_indicator: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            success_fg: Color::Numbered(15),
            success_bg: Color::Numbered(236),

            failure_fg: Color::Numbered(15),
            failure_bg: Color::Numbered(161),

            root_indicator: "#".into(),
            user_indicator: "\\$".into(),
        }
    }
}

impl ToSegment for CommandStatus {
    type Args = Args;
    type Theme = Theme;

    fn to_segment(
        _args: Option<Self::Args>,
        state: &ApplicationState,
    ) -> crate::Result<Vec<Segment>> {
        let theme = &state.theme.command_status;

        let (fg, bg) = match state.exit_code {
            0 => (theme.success_fg, theme.success_bg),
            _ => (theme.failure_fg, theme.failure_bg),
        };

        Ok(vec![Segment {
            bg,
            fg,
            separator: Separator::Thick,
            text: theme.user_indicator.clone(),
            source: "CommandStatus",
        }])
    }
}
