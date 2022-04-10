//! The `Git` segment displays information about a git repository
//!
//! This module provides the following segments that can be configured from the [`Args`] struct:
//! * branch
//! * ahead / behind remote
//! * staged items count
//! * modified items count
//! * untracked items count
//! * in-progress action (e.g. rebase, merge, cherry pick)
//! * stash count

use std::fs::read_to_string;

use anyhow::{anyhow, Context, Result};
use git2::{BranchType, ErrorCode, Repository, RepositoryState, StatusOptions};
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
pub struct Args {
    /// Show the git badge before the branch.  The badge itself can be configured via the
    /// [`vcs::Symbols`](`crate::segment::vcs::Symbols`) config object.
    ///
    /// **TODO** implement badges for well known remotes (e.g. GitHub, Bitbucket)
    pub show_vcs_badge: bool,

    /// Show count of stashed objects after the untracked badge.
    pub show_stash: bool,

    /// Whether to show a segment when there's an in-progress operation.  More granular options are below.
    pub show_in_progress: bool,

    /// Show a segment if we're bisecting.
    pub show_bisect: bool,

    /// Show a segment if we're picking cherries.
    pub show_cherry_pick: bool,

    /// Show a segment if we're mid-merge.
    pub show_merge: bool,

    /// Show a segment if we're in the middle of a rebase.
    pub show_rebase: bool,
}

/// High level statistics for the current git repo
struct Stats {
    /// Number of files with unstaged changes
    pub changed: usize,

    /// Number of files with conflicts
    pub conflicted: usize,

    /// Number of files staged for commit
    pub staged: usize,

    /// Number of untracked files
    pub untracked: usize,

    /// Number of stashes on the current repo
    pub stashed: usize,
}

fn seg_in_progress(repo: &Repository, args: &Args, theme: &VcsTheme, segments: &mut Vec<Segment>) {
    if !args.show_in_progress {
        return;
    }

    match repo.state() {
        RepositoryState::Bisect if args.show_bisect => segments.push(Segment {
            fg: theme.git_in_progress_fg,
            bg: theme.git_in_progress_bg,
            separator: Separator::Thick,
            text: "bisect".to_string(),
            source: "Git::Bisect",
        }),
        RepositoryState::CherryPick | RepositoryState::CherryPickSequence
            if args.show_cherry_pick =>
        {
            segments.push(Segment {
                fg: theme.git_in_progress_fg,
                bg: theme.git_in_progress_bg,
                separator: Separator::Thick,
                text: theme.symbols.cherry_pick.clone(),
                source: "Git::CherryPick",
            })
        }
        RepositoryState::Merge if args.show_merge => segments.push(Segment {
            fg: theme.git_in_progress_fg,
            bg: theme.git_in_progress_bg,
            separator: Separator::Thick,
            text: "merge".to_string(),
            source: "Git::Merge",
        }),
        state @ RepositoryState::Rebase
        | state @ RepositoryState::RebaseInteractive
        | state @ RepositoryState::RebaseMerge
            if args.show_rebase =>
        {
            let text = match state {
                RepositoryState::RebaseInteractive => {
                    // Well this is annoying
                    // https://github.com/libgit2/libgit2/issues/6127
                    let total_cmds = read_to_string(repo.path().join("rebase-merge/end")).unwrap();
                    let completed_cmds =
                        read_to_string(repo.path().join("rebase-merge/msgnum")).unwrap();
                    let total_cmds: usize = total_cmds.trim_end_matches('\n').parse().unwrap();
                    let completed_cmds: usize =
                        completed_cmds.trim_end_matches('\n').parse().unwrap();
                    format!("int rebase {}/{}", completed_cmds, total_cmds)
                }
                _ => "rebase".into(),
            };

            segments.push(Segment {
                fg: theme.git_in_progress_fg,
                bg: theme.git_in_progress_bg,
                separator: Separator::Thick,
                text,
                source: "Git::Rebase",
            })
        }
        _ => {}
    }
}

fn seg_ahead_behind(
    repo: &Repository,
    _args: &Args,
    theme: &VcsTheme,
    segments: &mut Vec<Segment>,
) -> Result<()> {
    if repo.head_detached()? {
        return Ok(());
    }

    let head = match repo.head() {
        Ok(head) => head,
        // This seems like the wrong way to do things, but in a repo with no commits this is normal
        Err(_) => return Ok(())
    };

    let head_name = head
        .shorthand()
        .ok_or_else(|| anyhow!("couldn't get a shorthand version of head"))?;

    let head_branch = repo.find_branch(head_name, BranchType::Local)?;

    let head_oid = head
        .target()
        .ok_or_else(|| anyhow!("couldn't find head -> target"))?;

    // On error: no upstream to track so we can't generate meaningful info.  But this is perfectly
    // normal on a new repo.
    let upstream_branch = match head_branch.upstream() {
        Ok(upstream_branch) => upstream_branch,
        Err(_) => return Ok(()),
    };
    
    let upstream_oid = upstream_branch
        .get()
        .target()
        .ok_or_else(|| anyhow!("couldn't find upstream oid"))?;

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

    Ok(())
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

fn seg_stashed(
    _repo: &Repository,
    stats: &Stats,
    args: &Args,
    theme: &VcsTheme,
    segments: &mut Vec<Segment>,
) {
    if stats.stashed > 0 && args.show_stash {
        segments.push(Segment {
            fg: theme.git_stashed_fg,
            bg: theme.git_stashed_bg,
            separator: Separator::Thick,
            text: format!("{}{}", stats.stashed, theme.symbols.stash),
            source: "Git::Stashed",
        })
    }
}

fn seg_current_branch(
    repo: &Repository,
    stats: &Stats,
    _args: &Args,
    theme: &VcsTheme,
    segments: &mut Vec<Segment>,
) -> Result<()> {
    let (fg, bg) = match stats.dirty() {
        false => (theme.repo_clean_fg, theme.repo_clean_bg),
        true => (theme.repo_dirty_fg, theme.repo_dirty_bg),
    };

    let head = match repo.head() {
        Ok(head) => Some(head),
        Err(ref e) if e.code() == ErrorCode::UnbornBranch || e.code() == ErrorCode::NotFound => {
            None
        }
        Err(e) => Err(e)?,
    };

    let head = head.as_ref().and_then(|h| h.shorthand());

    segments.push(Segment {
        bg,
        fg,
        separator: Separator::Thick,
        text: format!(
            "{} {}",
            theme.symbols.git,
            head.unwrap_or("HEAD (no branch)")
        ),
        source: "Git::Branch",
    });

    Ok(())
}

impl Default for Args {
    fn default() -> Self {
        Self {
            show_vcs_badge: true,
            show_stash: true,
            show_in_progress: true,
            show_bisect: true,
            show_cherry_pick: true,
            show_merge: true,
            show_rebase: true,
        }
    }
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

    fn error_context() -> &'static str {
        "segment::Git"
    }

    fn to_segment(
        args: Option<Self::Args>,
        state: &ApplicationState,
    ) -> crate::Result<Vec<Segment>> {
        let args = args.unwrap_or_default();

        let mut repo = match Repository::discover(".") {
            Ok(repo) => repo,
            Err(_) => return Ok(vec![]),
        };

        // Meh
        let mut stashed = 0;
        repo.stash_foreach(|_, _, _| {
            stashed += 1;
            true
        })?;

        let mut segments = vec![];

        if repo.is_bare() {
            return Err(anyhow!("Git segment doesn't work on bare repos"));
        }

        let mut opts = StatusOptions::new();
        opts.include_untracked(true)
            .recurse_untracked_dirs(false);
        let statuses = repo.statuses(Some(&mut opts))?;

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
            stashed,
        };

        seg_current_branch(&repo, &stats, &args, &state.theme.vcs, &mut segments)
            .context("seg_current_branch")
            .map_err(|err| eprintln!("Error in promptr: {:?}", err))
            .ok();
        seg_ahead_behind(&repo, &args, &state.theme.vcs, &mut segments)
            .context("seg_ahead_behind")
            .map_err(|err| eprintln!("Error in promptr: {:?}", err))
            .ok();
        seg_in_progress(&repo, &args, &state.theme.vcs, &mut segments);
        seg_staged(&repo, &stats, &args, &state.theme.vcs, &mut segments);
        seg_changed(&repo, &stats, &args, &state.theme.vcs, &mut segments);
        seg_untracked(&repo, &stats, &args, &state.theme.vcs, &mut segments);
        seg_stashed(&repo, &stats, &args, &state.theme.vcs, &mut segments);

        Ok(segments)
    }
}
