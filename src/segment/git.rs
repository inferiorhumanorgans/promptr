//! The `Git` segment displays information about a git repository

use anyhow::anyhow;
use git2::{BranchType, ErrorCode, Repository, StatusOptions};
use serde::Deserialize;

use crate::segment::vcs::Theme as VcsTheme;
use crate::segment::{Segment, ToSegment};
use crate::{ApplicationState, Separator};

pub struct Git {}

/// Arguments for the `Git` segment
///
/// **TODO** make a variety of things configurable here including which segments to display.
#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Args {}

/// High level statistics for the current git repo
struct Stats {
    pub changed: usize,
    pub conflicted: usize,
    pub staged: usize,
    pub untracked: usize,
}

fn seg_ahead_behind(
    repo: &Repository,
    _args: &Args,
    theme: &VcsTheme,
    segments: &mut Vec<Segment>,
) {
    let head = match repo.head() {
        Ok(head) => head,
        Err(_) => return,
    };
    let head_name = head.shorthand().unwrap();
    let head_branch = repo.find_branch(head_name, BranchType::Local).unwrap();
    let head_oid = head.target().unwrap();
    let upstream_branch = match head_branch.upstream() {
        Ok(upstream) => upstream,
        Err(_) => return, // No upstream to track so we can't generate meaningful info.
    };
    let upstream_oid = upstream_branch.get().target().unwrap();

    if let Ok((ahead, behind)) = repo.graph_ahead_behind(head_oid, upstream_oid) {
        let first_separator = if ahead > 0 && behind > 0 {
            Separator::Thin
        } else {
            Separator::Thick
        };

        if ahead > 0 {
            segments.push(Segment {
                bg: theme.git_ahead_bg,
                fg: theme.git_ahead_fg,
                separator: first_separator,
                text: format!("{}{}", ahead, theme.symbols.ahead),
                source: "Git::Ahead",
            });
        }

        if behind > 0 {
            segments.push(Segment {
                bg: theme.git_behind_bg,
                fg: theme.git_behind_fg,
                separator: Separator::Thick,
                text: format!("{}{}", behind, theme.symbols.behind),
                source: "Git::Behind",
            });
        }
    }
}

fn seg_untracked(
    _repo: &Repository,
    stats: &Stats,
    _args: &Args,
    theme: &VcsTheme,
    segments: &mut Vec<Segment>,
) {
    if stats.untracked > 0 {
        segments.push(Segment {
            fg: theme.git_untracked_fg,
            bg: theme.git_untracked_bg,
            separator: Separator::Thick,
            text: format!("{}{}", stats.untracked, theme.symbols.new),
            source: "Git::Untracked",
        });
    }
}

fn seg_changed(
    _repo: &Repository,
    stats: &Stats,
    _args: &Args,
    theme: &VcsTheme,
    segments: &mut Vec<Segment>,
) {
    if stats.changed > 0 {
        segments.push(Segment {
            fg: theme.git_changed_fg,
            bg: theme.git_changed_bg,
            separator: Separator::Thick,
            text: format!("{}{}", stats.changed, theme.symbols.changed),
            source: "Git::Changed",
        });
    }
}

fn seg_staged(
    _repo: &Repository,
    stats: &Stats,
    _args: &Args,
    theme: &VcsTheme,
    segments: &mut Vec<Segment>,
) {
    if stats.staged > 0 {
        segments.push(Segment {
            fg: theme.git_staged_fg,
            bg: theme.git_staged_bg,
            separator: Separator::Thick,
            text: format!("{}+", stats.staged),
            source: "Git::Staged",
        });
    }
}

fn seg_current_branch(
    repo: &Repository,
    stats: &Stats,
    _args: &Args,
    theme: &VcsTheme,
    segments: &mut Vec<Segment>,
) {
    let (fg, bg) = match stats.dirty() {
        false => (theme.repo_clean_fg, theme.repo_clean_bg),
        true => (theme.repo_dirty_fg, theme.repo_dirty_bg),
    };

    let head = match repo.head() {
        Ok(head) => Some(head),
        Err(ref e) if e.code() == ErrorCode::UnbornBranch || e.code() == ErrorCode::NotFound => {
            None
        }
        Err(_) => return,
    };

    let head = head.as_ref().and_then(|h| h.shorthand());

    segments.push(Segment {
        bg,
        fg,
        separator: Separator::Thick,
        text: head
            .or(Some("HEAD (no branch)"))
            .map(|x| format!("{} {}", theme.symbols.git, x))
            .unwrap(),
        source: "Git::Branch",
    });
}

impl Stats {
    /// Returns true if there are local modifications, conflicts, staged, or new files
    pub fn dirty(&self) -> bool {
        let filth = self.changed + self.conflicted + self.staged + self.untracked;

        filth > 0
    }
}

impl ToSegment for Git {
    type Args = Args;
    type Theme = super::vcs::Theme;

    fn to_segment(
        args: Option<Self::Args>,
        state: &ApplicationState,
    ) -> crate::Result<Vec<Segment>> {
        let args = args.unwrap_or_default();

        let repo = Repository::open(".")?;

        let mut segments = vec![];

        if repo.is_bare() {
            return Err(anyhow!("Git segment doesn't work on bare repos"));
        }

        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .include_untracked(true)
            .recurse_untracked_dirs(true);
        let statuses = repo.statuses(Some(&mut opts)).unwrap();

        let untracked = statuses
            .iter()
            .filter(|e| e.status() == git2::Status::WT_NEW)
            .count();

        let staged = statuses
            .iter()
            .filter(|e| {
                let status = e.status();

                status.contains(git2::Status::INDEX_NEW)
                    || status.contains(git2::Status::INDEX_MODIFIED)
                    || status.contains(git2::Status::INDEX_DELETED)
                    || status.contains(git2::Status::INDEX_RENAMED)
                    || status.contains(git2::Status::INDEX_TYPECHANGE)
            })
            .count();

        let changed = statuses
            .iter()
            .filter(|e| {
                let status = e.status();

                status.contains(git2::Status::WT_MODIFIED)
                    || status.contains(git2::Status::WT_DELETED)
                    || status.contains(git2::Status::WT_RENAMED)
                    || status.contains(git2::Status::WT_TYPECHANGE)
            })
            .count();

        let conflicted = 0;

        let stats = Stats {
            changed,
            conflicted,
            staged,
            untracked,
        };

        seg_current_branch(&repo, &stats, &args, &state.theme.vcs, &mut segments);
        seg_ahead_behind(&repo, &args, &state.theme.vcs, &mut segments);
        seg_staged(&repo, &stats, &args, &state.theme.vcs, &mut segments);
        seg_changed(&repo, &stats, &args, &state.theme.vcs, &mut segments);
        seg_untracked(&repo, &stats, &args, &state.theme.vcs, &mut segments);

        Ok(segments)
    }
}
