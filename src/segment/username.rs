//! The `Username` segment
use std::env;

use serde::{Deserialize, Serialize};

use crate::segment::{Segment, ToSegment};
use crate::{ApplicationState, Color, Separator};

pub struct Username {}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Args {}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Theme {
    pub fg: Color,
    pub bg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            fg: Color::Numbered(250),
            bg: Color::Numbered(240),
        }
    }
}

impl ToSegment for Username {
    type Args = Args;
    type Theme = Theme;

    fn to_segment(_args: Option<Self::Args>, state: &ApplicationState) -> crate::Result<Vec<Segment>> {
        let theme = &state.theme.username;

        let text = env::var("USER").expect("Couldn't determine username");

        Ok(vec![Segment {
            fg: theme.fg,
            bg: theme.bg,
            separator: Separator::Thick,
            text,
            source: "Username",
        }])
    }
}
