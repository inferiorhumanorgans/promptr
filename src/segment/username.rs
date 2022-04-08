//! The `Username` segment displays the current username and provides a `sudo` indicator
//!
//! **TODO** check `${SUDO_UID}` to determine if we're being run in a `sudo` context

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use crate::ansi::Color;
use crate::segment::{Segment, ToSegment};
use crate::{ApplicationState, Separator};

pub struct Username {}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SudoIndicator {
    Symbol,
    Username,
    None,
}

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Args {
    pub sudo_indicator: SudoIndicator,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Theme {
    pub fg: Color,
    pub bg: Color,

    pub sudo_indicator: String,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            sudo_indicator: SudoIndicator::Symbol,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            fg: Color::Numbered(250),
            bg: Color::Numbered(240),

            // ≈ pseudo… almost equal
            sudo_indicator: "\u{2248}".into(),
        }
    }
}

impl ToSegment for Username {
    type Args = Args;
    type Theme = Theme;

    fn to_segment(
        args: Option<Self::Args>,
        state: &ApplicationState,
    ) -> crate::Result<Vec<Segment>> {
        let args = args.unwrap_or_default();

        let theme = &state.theme.username;

        let effective_user = state
            .env
            .get("USER")
            .ok_or_else(|| anyhow!("$USER not set"))?
            .to_string();

        let sudo_user = state.env
            .get("SUDO_USER");

        let text = match sudo_user {
            None => effective_user,
            Some(sudo_user) => {
                match args.sudo_indicator {
                    SudoIndicator::None => effective_user,
                    SudoIndicator::Symbol => format!("{}{}", theme.sudo_indicator, effective_user),
                    SudoIndicator::Username => format!("{} \u{2192} {}", sudo_user, effective_user),
                }
            }
        };

        Ok(vec![Segment {
            fg: theme.fg,
            bg: theme.bg,
            separator: Separator::Thick,
            text,
            source: "Username",
        }])
    }
}
