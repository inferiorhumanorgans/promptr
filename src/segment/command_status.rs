use serde::{Deserialize, Serialize};

use crate::segment::{Segment, ToSegment};
use crate::{ApplicationState, Color, Separator};

pub struct CommandStatus {}

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Args {
    root_indicator: String,
    user_indicator: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Theme {
    pub success_fg: Color,
    pub success_bg: Color,

    pub failure_fg: Color,
    pub failure_bg: Color,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            root_indicator: "#".into(),
            user_indicator: "\\$".into(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            success_fg: Color::Numbered(15),
            success_bg: Color::Numbered(236),
            failure_fg: Color::Numbered(15),
            failure_bg: Color::Numbered(161),
        }
    }
}

impl ToSegment for CommandStatus {
    type Args = Args;
    type Theme = Theme;

    fn to_segment(args: Option<Self::Args>, state: &ApplicationState) -> crate::Result<Vec<Segment>> {
        let args = args.unwrap_or_default();

        let theme = &state.theme.command_status;

        let (fg, bg) = match state.exit_code {
            0 => (theme.success_fg, theme.success_bg),
            _ => (theme.failure_fg, theme.failure_bg),
        };

        Ok(vec![Segment {
            bg,
            fg,
            separator: Separator::Thick,
            text: args.user_indicator,
            source: "CommandStatus",
        }])
    }
}
