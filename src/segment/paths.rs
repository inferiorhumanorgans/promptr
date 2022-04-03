//! The `Path` segment displays breadcrumbs to the current working directory
use std::env;
use std::path::Component;
use std::str::FromStr;

use itertools::{Itertools, Position};
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::segment::{Segment, ToSegment};
use crate::{ApplicationState, Color, Separator};

pub struct Paths {}

/// Argumnts for the `Paths` segment.
#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Args {
    /// String/icon to replace the home directory component.  Grey beards probably want a tilde.
    pub home_dir_replacement: String,

    /// Whether or not to show a path segment for the root directory
    pub show_root: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Theme {
    pub fg: Color,
    pub bg: Color,

    pub home_fg: Color,
    pub home_bg: Color,

    pub last_fg: Color,
    pub last_bg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            fg: Color::Numbered(250),
            bg: Color::Numbered(237),

            home_fg: Color::Numbered(15),
            home_bg: Color::Numbered(31),

            last_fg: Color::Numbered(254),
            last_bg: Color::Numbered(237),
        }
    }
}

impl Default for Args {
    fn default() -> Self {
        Self {
            home_dir_replacement: Paths::HOME_SHORTENED.into(),
            show_root: false,
        }
    }
}

impl Paths {
    const HOME_SHORTENED: &'static str = "~";
}

impl ToSegment for Paths {
    type Args = Args;
    type Theme = Theme;

    fn to_segment(
        args: Option<Self::Args>,
        state: &ApplicationState,
    ) -> crate::Result<Vec<Segment>> {
        let args = args.unwrap_or_default();
        let theme = &state.theme.paths;

        let path = env::var("PWD")?;
        let home_dir = env::var("HOME").expect("Couldn't determine home directory");
        let home_regex = Regex::new(format!("^{}", home_dir).as_str()).unwrap();
        let path: String = home_regex
            .replace(path.as_ref(), Self::HOME_SHORTENED)
            .into();
        let path = std::path::PathBuf::from_str(path.as_str()).unwrap();
        let segments = path
            .components()
            .with_position()
            .filter_map(|component| match component {
                Position::First(Component::RootDir) => match args.show_root {
                    false => None,
                    true => Some(Segment {
                        fg: theme.fg,
                        bg: theme.bg,
                        separator: Separator::Thin,
                        text: "/".into(),
                        source: "Paths::First::Root",
                    }),
                },
                Position::First(Component::Normal(p)) => {
                    if p == Self::HOME_SHORTENED {
                        Some(Segment {
                            fg: theme.home_fg,
                            bg: theme.home_bg,
                            separator: Separator::Thick,
                            text: args.home_dir_replacement.clone(),
                            source: "Paths::First::Home",
                        })
                    } else {
                        Some(Segment {
                            fg: theme.fg,
                            bg: theme.bg,
                            separator: Separator::Thin,
                            text: p.to_string_lossy().into(),
                            source: "Paths::First::Normal",
                        })
                    }
                }
                Position::First(_) => None,
                Position::Only(Component::Normal(p)) => {
                    if p == Self::HOME_SHORTENED {
                        Some(Segment {
                            fg: theme.home_fg,
                            bg: theme.home_bg,
                            separator: Separator::Thick,
                            text: args.home_dir_replacement.clone(),
                            source: "Paths::Only::Home",
                        })
                    } else {
                        Some(Segment {
                            fg: theme.fg,
                            bg: theme.bg,
                            separator: Separator::Thick,
                            text: p.to_string_lossy().into(),
                            source: "Paths::Only::Normal",
                        })
                    }
                }
                Position::Middle(Component::Normal(p)) => Some(Segment {
                    fg: theme.fg,
                    bg: theme.bg,
                    separator: Separator::Thin,
                    text: p.to_string_lossy().into(),
                    source: "Paths::Middle::Normal",
                }),
                Position::Last(Component::Normal(p)) => Some(Segment {
                    fg: theme.last_fg,
                    bg: theme.last_bg,
                    separator: Separator::Thick,
                    text: p.to_string_lossy().into(),
                    source: "Paths::Last::Normal",
                }),
                _ => None,
            })
            .collect();
        Ok(segments)
    }
}
