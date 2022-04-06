//! The `Screen` segment indicates if we are in a GNU Screen session and optionally the window
//! number and name of the screen
//!
//! **NOTE** Versions of GNU screen before 4.2.0 will not properly display emoji.  If you *are*
//! using an older version of screen ensure that you're using compatible icons everywhere.
//!
//! <https://unix.stackexchange.com/questions/81923/gnu-screen-doesnt-echo-unicode-characters-correct#answer-605566>

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

use crate::ansi::Color;
use crate::segment::{Segment, ToSegment};
use crate::{ApplicationState, Separator};

pub struct Screen {}

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Args {
    pub show_screen_icon: bool,

    pub show_screen_name: bool,
    pub show_screen_pid: bool,

    pub show_window_number: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Theme {
    /// Foreground color
    pub fg: Color,

    /// Background color
    pub bg: Color,

    /// Icon to display if we're inside a screen session
    pub screen_symbol: String,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            show_screen_icon: true,

            show_screen_name: true,
            show_screen_pid: false,

            show_window_number: true,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            fg: Color::Numbered(250),
            bg: Color::Numbered(238),

            // 📺
            screen_symbol: "\u{1f4fa}".into(),
        }
    }
}

impl ToSegment for Screen {
    type Args = Args;
    type Theme = Theme;

    fn to_segment(
        args: Option<Self::Args>,
        state: &ApplicationState,
    ) -> crate::Result<Vec<Segment>> {
        let args = args.unwrap_or_default();

        let Theme { fg, bg, .. } = state.theme.screen;
        let theme = &state.theme.screen;

        // This isn't reliable and will be lost if we sudo within the screen
        let screen_tty = match state.env.get("STY") {
            Some(sty) => sty,
            None => return Ok(vec![]),
        };

        let window = match state.env.get("WINDOW") {
            Some(window) => window,
            None => return Ok(vec![]),
        };

        let mut iterator = screen_tty.splitn(2, '.');
        let pid = iterator
            .next()
            .ok_or_else(|| anyhow!("couldn't parse $STY"))?;
        let name = iterator
            .next()
            .ok_or_else(|| anyhow!("couldn't parse $STY"))?;

        let text = format!(
            "{}{}{}{}{}{}",
            match args.show_window_number {
                true => window,
                false => "",
            },
            match args.show_window_number && (args.show_screen_pid || args.show_screen_name) {
                true => "[",
                false => "",
            },
            match args.show_screen_pid {
                true => format!("{}.", pid),
                false => "".into(),
            },
            match args.show_screen_name {
                true => name,
                false => "",
            },
            match args.show_window_number && (args.show_screen_pid || args.show_screen_name) {
                true => "]",
                false => "",
            },
            match args.show_screen_icon {
                true => format!(" {}", theme.screen_symbol),
                false => "".into(),
            },
        );

        Ok(vec![Segment {
            bg,
            fg,
            separator: Separator::Thick,
            text,
            source: "Screen",
        }])
    }
}
