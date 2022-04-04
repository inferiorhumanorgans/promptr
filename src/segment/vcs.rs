//! There are no segments here, just theme related structs.

use serde::{Deserialize, Serialize};

use crate::ansi::Color;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Symbols {
    pub detached: String,
    pub ahead: String,
    pub behind: String,
    pub staged: String,
    pub changed: String,
    pub new: String,
    pub conflicted: String,
    pub stash: String,

    pub git: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Theme {
    pub git_ahead_fg: Color,
    pub git_ahead_bg: Color,

    pub git_behind_fg: Color,
    pub git_behind_bg: Color,

    pub git_staged_fg: Color,
    pub git_staged_bg: Color,

    pub git_changed_fg: Color,
    pub git_changed_bg: Color,

    pub git_untracked_fg: Color,
    pub git_untracked_bg: Color,

    pub git_conflict_fg: Color,
    pub git_conflict_bg: Color,

    pub git_in_progress_fg: Color,
    pub git_in_progress_bg: Color,

    pub git_stashed_fg: Color,
    pub git_stashed_bg: Color,

    pub repo_clean_fg: Color,
    pub repo_clean_bg: Color,

    pub repo_dirty_fg: Color,
    pub repo_dirty_bg: Color,

    pub symbols: Symbols,
}

impl Default for Symbols {
    fn default() -> Self {
        Self {
            detached: "\u{2693}".into(),
            ahead: "\u{2B06}".into(),
            behind: "\u{2B07}".into(),
            staged: "\u{2714}".into(),
            changed: "✎".into(),
            new: "?".into(),
            conflicted: "\u{273C}".into(),
            stash: "\u{2398}".into(),

            git: "\u{E0A0}".into(),
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            git_ahead_fg: Color::Numbered(250),
            git_ahead_bg: Color::Numbered(240),

            git_behind_fg: Color::Numbered(250),
            git_behind_bg: Color::Numbered(240),

            git_staged_fg: Color::Numbered(15),
            git_staged_bg: Color::Numbered(22),

            git_changed_fg: Color::Numbered(15),
            git_changed_bg: Color::Numbered(130),

            git_untracked_fg: Color::Numbered(15),
            git_untracked_bg: Color::Numbered(52),

            git_conflict_fg: Color::Numbered(15),
            git_conflict_bg: Color::Numbered(9),

            git_in_progress_fg: Color::Numbered(15),
            git_in_progress_bg: Color::Numbered(208),

            git_stashed_fg: Color::Numbered(0),
            git_stashed_bg: Color::Numbered(221),
        
            repo_clean_fg: Color::Numbered(0),
            repo_clean_bg: Color::Numbered(148),

            repo_dirty_fg: Color::Numbered(15),
            repo_dirty_bg: Color::Numbered(161),

            symbols: Symbols::default(),
        }
    }
}
