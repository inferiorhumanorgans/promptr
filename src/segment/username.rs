//! The `Username` segment displays the current username and provides a `sudo` indicator

use anyhow::anyhow;
use serde::{Deserialize, Serialize, Serializer, ser::SerializeStruct};

use crate::ansi::Color;
use crate::segment::{Segment, ToSegment};
use crate::{ApplicationState, Separator};
use promptr_macros::SerializeNonDefault;

pub struct Username {}

/// The format in which we would like sudo shells to be represented
#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SudoIndicator {
    /// `≈ effective_user`
    Symbol,
    /// `user → effective user`
    Username,
    /// `effective_user`
    None,
}

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Args {
    pub sudo_indicator: SudoIndicator,
}

#[derive(Clone, Debug, Deserialize, PartialEq, SerializeNonDefault)]
#[serde(default, deny_unknown_fields)]
pub struct Theme {
    pub fg: Color,
    pub bg: Color,

    /// Prefix to the username if `Args.sudo_indicator` is set to `Symbol`
    pub sudo_indicator: String,

    /// Separator between the user and effective user if `Args.sudo_indicator` is set to `Username`
    pub sudo_separator: String,
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
            // user → effective user
            sudo_separator: "\u{2192}".into(),
        }
    }
}

impl ToSegment for Username {
    type Args = Args;
    type Theme = Theme;

    fn error_context() -> &'static str {
        "segment::Username"
    }

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

        let sudo_user = state.env.get("SUDO_USER");

        let text = match sudo_user {
            None => effective_user,
            Some(sudo_user) => match args.sudo_indicator {
                SudoIndicator::None => effective_user,
                SudoIndicator::Symbol => format!("{}{}", theme.sudo_indicator, effective_user),
                SudoIndicator::Username => {
                    format!("{} {} {}", sudo_user, theme.sudo_separator, effective_user)
                }
            },
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
