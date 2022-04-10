//! The `Path` segment displays breadcrumbs to the current working directory
use std::path::Component;
use std::str::FromStr;

use anyhow::anyhow;
use itertools::{Itertools, Position};
use regex::Regex;
use serde::{ser::SerializeStruct, Deserialize, Serialize, Serializer};

use crate::ansi::Color;
use crate::segment::{Segment, ToSegment};
use crate::{ApplicationState, Separator};
use promptr_macros::SerializeNonDefault;

pub struct Path {}

/// Arguments for the `Path` segment.
#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Args {
    /// Whether or not to show a path segment for the root directory
    pub show_root: bool,

    /// Add a leading segment if there's more than one directory in the [stack](https://www.gnu.org/software/bash/manual/html_node/The-Directory-Stack.html)
    pub show_dir_stack: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, SerializeNonDefault)]
#[serde(default, deny_unknown_fields)]
pub struct Theme {
    pub fg: Color,
    pub bg: Color,

    pub home_fg: Color,
    pub home_bg: Color,

    pub last_fg: Color,
    pub last_bg: Color,

    pub dir_stack_indicator: String,

    /// String/icon to replace the home directory component.  Grey beards probably want a tilde.
    pub home_dir_replacement: String,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            show_root: false,
            show_dir_stack: true,
        }
    }
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

            // 📚 – *stack* of books
            dir_stack_indicator: "\u{1f4da}".into(),
            home_dir_replacement: Path::HOME_SHORTENED.into(),
        }
    }
}

impl Path {
    const HOME_SHORTENED: &'static str = "~";
}

impl ToSegment for Path {
    type Args = Args;
    type Theme = Theme;

    fn error_context() -> &'static str {
        "segment::Path"
    }

    fn to_segment(
        args: Option<Self::Args>,
        state: &ApplicationState,
    ) -> crate::Result<Vec<Segment>> {
        let args = args.unwrap_or_default();

        let theme = &state.theme.path;

        let path = state
            .env
            .get("PWD")
            .ok_or_else(|| anyhow!("Couldn't determine current directory, $PWD not set"))?
            .to_string();
        let home_dir = state
            .env
            .get("HOME")
            .ok_or_else(|| anyhow!("Couldn't determine home directory, $HOME not set"))?
            .to_string();
        let home_regex = Regex::new(format!("^{}", home_dir).as_str())?;
        let path: String = home_regex
            .replace(path.as_ref(), Self::HOME_SHORTENED)
            .into();
        let path = std::path::PathBuf::from_str(path.as_str())?;
        let mut segments: Vec<Segment> = path
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
                        source: "Path::First::Root",
                    }),
                },
                Position::First(Component::Normal(p)) => {
                    if p == Self::HOME_SHORTENED {
                        Some(Segment {
                            fg: theme.home_fg,
                            bg: theme.home_bg,
                            separator: Separator::Thick,
                            text: theme.home_dir_replacement.clone(),
                            source: "Path::First::Home",
                        })
                    } else {
                        Some(Segment {
                            fg: theme.fg,
                            bg: theme.bg,
                            separator: Separator::Thin,
                            text: p.to_string_lossy().into(),
                            source: "Path::First::Normal",
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
                            text: theme.home_dir_replacement.clone(),
                            source: "Path::Only::Home",
                        })
                    } else {
                        Some(Segment {
                            fg: theme.fg,
                            bg: theme.bg,
                            separator: Separator::Thick,
                            text: p.to_string_lossy().into(),
                            source: "Path::Only::Normal",
                        })
                    }
                }
                Position::Middle(Component::Normal(p)) => Some(Segment {
                    fg: theme.fg,
                    bg: theme.bg,
                    separator: Separator::Thin,
                    text: p.to_string_lossy().into(),
                    source: "Path::Middle::Normal",
                }),
                Position::Last(Component::Normal(p)) => Some(Segment {
                    fg: theme.last_fg,
                    bg: theme.last_bg,
                    separator: Separator::Thick,
                    text: p.to_string_lossy().into(),
                    source: "Path::Last::Normal",
                }),
                _ => None,
            })
            .collect();

        if args.show_dir_stack {
            if let Some(dirs) = state.env.get("dirs") {
                let dir_stack_depth = dirs.split('\n').count();
                if dir_stack_depth > 1 {
                    segments.insert(
                        0,
                        Segment {
                            fg: theme.fg,
                            bg: theme.bg,
                            separator: Separator::Thick,
                            text: format!("{} {}", dir_stack_depth, theme.dir_stack_indicator),
                            source: "Path::BashDirStack",
                        },
                    );
                }
            }
        }

        Ok(segments)
    }
}
