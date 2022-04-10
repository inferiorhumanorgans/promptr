use std::fs::File;
use std::io::{BufReader, Cursor};

use lzma_rs::xz_decompress;
use tar::Archive;
use tempfile::{tempdir, TempDir};

use crate::segment::{git::Git, ToSegment};
use crate::test::segment::declare_segement_test;
use crate::test::AppEnv;
use crate::{ApplicationState, Theme};

declare_segement_test!([]);

fn get_testcase_from_tarball(name: &'static str, state: &mut ApplicationState) -> TempDir {
    let temp_dir = tempdir().expect("Failed to create temporary directory");

    let tar_path = format!("git-tests/{}.tar.xz", name);
    let mut tar_expanded = vec![];
    let mut tar_file = BufReader::new(File::open(tar_path).unwrap());
    xz_decompress(&mut tar_file, &mut tar_expanded).unwrap();

    let mut tarball = Archive::new(Cursor::new(&tar_expanded));
    tarball.unpack(&temp_dir).unwrap();

    let repo_path = temp_dir.path().join(name).to_string_lossy().into();
    state
        .env
        .insert(String::from("__PROMPTR_GIT_REPO"), repo_path);

    temp_dir
}

segment_test! {
    fn empty_git_repo() {
        |args, mut state : ApplicationState| {
            let _temp_dir = get_testcase_from_tarball("empty", &mut state);

            {
                let mut state = state.clone();
                let mut theme = state.theme.clone();
                theme.vcs.symbols.git = "".to_string();
                state.theme = &theme;

                let segments = Git::to_segment_generic(args, &state).unwrap();

                assert_eq!(1, segments.len());

                assert_eq!(
                    crate::segment::Segment {
                        bg: theme.vcs.repo_clean_bg,
                        fg: theme.vcs.repo_clean_fg,
                        text: String::from(" master (unborn)"),
                        separator: crate::Separator::Thick,
                        source: "Git::Branch",
                    },
                    segments[0]
                );
            }
        }
    }
}

segment_test! {
    fn untracked_file() {
        |args, mut state : ApplicationState| {
            let _temp_dir = get_testcase_from_tarball("untracked-file", &mut state);

            {
                let mut state = state.clone();
                let mut theme = state.theme.clone();
                theme.vcs.symbols.git = "".to_string();
                state.theme = &theme;

                let segments = Git::to_segment_generic(args, &state).unwrap();

                assert_eq!(2, segments.len());

                assert_eq!(
                    crate::segment::Segment {
                        bg: theme.vcs.repo_dirty_bg,
                        fg: theme.vcs.repo_dirty_fg,
                        text: String::from(" master"),
                        separator: crate::Separator::Thick,
                        source: "Git::Branch",
                    },
                    segments[0]
                );

                assert_eq!(
                    crate::segment::Segment {
                        bg: theme.vcs.git_untracked_bg,
                        fg: theme.vcs.git_untracked_fg,
                        text: String::from("1?"),
                        separator: crate::Separator::Thick,
                        source: "Git::Untracked",
                    },
                    segments[1]
                );
            }
        }
    }
}

segment_test! {
    fn rebase_interactive() {
        |args, mut state : ApplicationState| {
            let _temp_dir = get_testcase_from_tarball("rebase-interactive", &mut state);

            {
                let mut state = state.clone();
                let mut theme = state.theme.clone();
                theme.vcs.symbols.git = "".to_string();
                state.theme = &theme;

                let segments = Git::to_segment_generic(args, &state).unwrap();

                eprintln!("Segments: {:#?}", segments);
                assert_eq!(2, segments.len());

                assert_eq!(
                    crate::segment::Segment {
                        bg: theme.vcs.repo_clean_bg,
                        fg: theme.vcs.repo_clean_fg,
                        text: String::from(" master"),
                        separator: crate::Separator::Thick,
                        source: "Git::Branch",
                    },
                    segments[0]
                );

                assert_eq!(
                    crate::segment::Segment {
                        bg: theme.vcs.git_in_progress_bg,
                        fg: theme.vcs.git_in_progress_fg,
                        text: String::from("int rebase 2/3"),
                        separator: crate::Separator::Thick,
                        source: "Git::Rebase",
                    },
                    segments[1]
                );
            }
        }
    }
}
segment_test! {
    fn cherry_pick_interactive() {
        |args, mut state : ApplicationState| {
            let _temp_dir = get_testcase_from_tarball("cherry-pick", &mut state);

            // Scope for fun and profit
            {
                let mut state = state.clone();
                let mut theme = state.theme.clone();
                theme.vcs.symbols.git = "".to_string();
                theme.vcs.symbols.cherry_pick = "[CHERRY_PICKING]".to_string();
                state.theme = &theme;

                let segments = Git::to_segment_generic(args, &state).unwrap();

                eprintln!("Segments: {:#?}", segments);
                assert_eq!(2, segments.len());

                assert_eq!(
                    crate::segment::Segment {
                        bg: theme.vcs.repo_clean_bg,
                        fg: theme.vcs.repo_clean_fg,
                        text: String::from(" master"),
                        separator: crate::Separator::Thick,
                        source: "Git::Branch",
                    },
                    segments[0]
                );

                assert_eq!(
                    crate::segment::Segment {
                        bg: theme.vcs.git_in_progress_bg,
                        fg: theme.vcs.git_in_progress_fg,
                        text: String::from("[CHERRY_PICKING]"),
                        separator: crate::Separator::Thick,
                        source: "Git::CherryPick",
                    },
                    segments[1]
                );
            }
        }
    }
}
