//! The `CommandStatus` segment displays the root / non-privileged indicator and the last exit value

use serde::{Deserialize, Serialize, Serializer, ser::SerializeStruct};

use crate::ansi::Color;
use crate::segment::{Segment, ToSegment};
use crate::{ApplicationState, Separator};
use promptr_macros::SerializeNonDefault;

pub struct CommandStatus {}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Args {}

/// Theme for the [`CommandStatus`] segment.
///
/// TODO: Make the exit status coloring optional
#[derive(Clone, Debug, Deserialize, PartialEq, SerializeNonDefault)]
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

    fn error_context() -> &'static str {
        "segment::CommandStatus"
    }

    fn to_segment(
        _args: Option<Self::Args>,
        state: &ApplicationState,
    ) -> crate::Result<Vec<Segment>> {
        let theme = &state.theme.command_status;

        let exit_code = state.env.get("code").map_or("0", String::as_str);
        let (fg, bg) = match exit_code.parse::<u8>() {
            Ok(0) => (theme.success_fg, theme.success_bg),
            Ok(_) => (theme.failure_fg, theme.failure_bg),
            _ => (theme.success_fg, theme.success_bg),
        };

        let uid = state.env.get("uid").map_or("65535", String::as_str);

        let text = match uid.parse::<u32>() {
            Ok(0) => theme.root_indicator.clone(),
            _ => theme.user_indicator.clone(),
        };

        Ok(vec![Segment {
            bg,
            fg,
            separator: Separator::Thick,
            text,
            source: "CommandStatus",
        }])
    }
}
