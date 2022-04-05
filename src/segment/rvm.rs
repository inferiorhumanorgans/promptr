//! The `Rvm` segment displays the the current gemset if RVM is loaded
//!
//! This is a a very rough reinterpretation of how `rvm` does things.  Unfortunately `rvm` uses
//! a huge mess of shell scripts which are, by their nature, not performant.  As this can be a
//! bit messy it is not enabled in the default configuration and is gated by the `segment-rvm`
//! feature.
//!
//! ### Our intended logic
//!
//! * Assume a rubie is specified as `[interpreter]ruby_veresion[@gemset]`, `interpreter` defaults to `ruby` and `gemset` to `default`.  See [`Gemset`] for details.
//! * Load environment required variables, bail if any are not present
//!     + `rvm_version` — this is a proxy for whether `rvm` is active
//!     + `PWD` and `HOME` — as we'll need them later.
//!     + `rvm_path` — this is where `rvm` supposedly lives
//! * Determine if we should show rvm info
//!     + if [`args.force_show`][`Args`] is `true`, always print `rvm`
//!     + Otherwise search for a file named `Gemfile` in the current and ancestor directories
//!         + Stop if we reach `$HOME`
//!         + Stop if we reach `rvm_path` (ew)
//!         + Stop if we find `Gemfile`
//! * Recurse through our ancestors looking for `.ruby-version`
//! * `$GEM_HOME` defines where the current gemset lives.  Subtract `$rvm_path` to get the name.  Parse it as an actual [`semver::Version`]
//! * If `.ruby-version` exists parse it with the version as a [`semver::VersionReq`]
//!     + Ensure that the requesetd and actual / current interpreters match
//!     + The requested and actual / current versions match
//!     + TODO: check the gemsets
//! * If `.ruby-version` (requested) doesn't match `$GEM_HOME` (actual / current)
//!     + `rvm` has likely printed a warning
//!     + Print the current ruby version and append [`theme.rvm.mismatch_symbol`](`Theme`)
//! * If the two match, print the current ruby version

use std::env;
use std::fs::metadata;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use anyhow::anyhow;
use regex::Regex;
use serde::{Deserialize, Serialize};

use crate::ansi::Color;
use crate::segment::{Segment, ToSegment};
use crate::{ApplicationState, Separator};

pub struct Rvm {}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Args {
    /// Show even if there's no `Gemset` file in the current or ancestor directories
    pub force_show: bool,
}


/// An RVM "rubie" – a specific instance of a ruby environment
///
/// There are three components: an interpreter (default: `ruby`), a version, and an optional gemset.
/// These are the Ruby interpreters that RVM supports as of April 2022:
/// * ruby - MRI ruby (The Gold Standard)
/// * ironruby - a .NET ruby
/// * jruby - Java implementation of the ruby
/// * macruby - implementation of ruby 1.9 directly on top of macOS core technologies
/// * maglev - 64-bit implementation on top of VMware's GemStone
/// * mruby - lightweight ruby
/// * opal - ruby to JavaScript compiler
/// * rbx - Rubinius - a next generation virtual machine VM for ruby
/// * topaz - high performance ruby, written in RPython
/// * truffleruby - high performance ruby using GraalVM
///
/// TODO: Rename this to Rubie
/// TODO: Toggle whether to show the interpreter even if it's non-default
#[derive(Clone, PartialEq)]
struct Gemset<SemType: std::fmt::Debug + FromStr> {
    pub interp: String,
    pub version: SemType,
    pub gemset: Option<String>,
}

/// Theme for the [`Rvm`] segment, located at `theme.rvm` in the [`configuration file`](`crate::PromptrConfig`)
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Theme {
    /// Foreground color for the [`Rvm`] segment
    pub fg: Color,

    /// Background color for the [`Rvm`] segment
    pub bg: Color,

    /// Appended if we suspect [`Rvm`] can't find the desired rubie
    pub mismatch_symbol: String,
}

fn find_ancestors(target: &str, pwd: &str, home: &str, rvm_path: &PathBuf) -> Option<PathBuf> {
    let mut has_target = None;
    let mut path = Some(Path::new(pwd));

    let home_path = Path::new(home);

    // This is loosely based on scripts/functions/rvmrc_project
    while path.is_some() {
        let path_ref = path.as_ref().unwrap();
        let file_ref = path_ref.join(target);

        if path_ref != &home_path && path_ref != rvm_path {
            if metadata(&file_ref).is_ok() {
                has_target = Some(file_ref);
                break;
            }
        }

        path = path_ref.parent();
    }
    has_target
}

impl<SemType> FromStr for Gemset<SemType>
where
    SemType: std::fmt::Debug + FromStr,
    <SemType as FromStr>::Err: std::fmt::Debug,
{
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let re = Regex::new(r"(([\w_]+)-)?(((\d+)\.)?((\d+)\.)?(\d+))(@([\w_]+))?")
            .expect("rvm regex error, we should never be here");
        if !re.is_match(s) {
            // Not a valid thing
            Err(())?
        }

        match re.captures(s) {
            None => Err(()),
            Some(caps) => {
                let version = SemType::from_str(caps.get(3).unwrap().as_str()).unwrap();

                Ok(
                    Self {
                        interp: caps
                            .get(2)
                            .map(|re_match| re_match.as_str().into())
                            .unwrap_or_else(|| "ruby".to_string()),
                        version,
                        gemset: caps.get(10).map(|re_match| re_match.as_str().to_string()),
                    }
                )
            }
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            fg: Color::Numbered(15),
            bg: Color::Numbered(124),
            // ≠ - not equal
            mismatch_symbol: " \u{2260}".to_string(),
        }
    }
}

impl ToSegment for Rvm {
    type Args = Args;
    type Theme = Theme;

    fn to_segment(
        args: Option<Self::Args>,
        state: &ApplicationState,
    ) -> crate::Result<Vec<Segment>> {
        let args = args.unwrap_or_default();

        let theme = &state.theme.rvm;

        // We don't actually care what the RVM version is
        // we just want to bail if RVM isn't loaded
        env::var("rvm_version")?;

        // Take a quick look for a Gemfile as a proxy for whether or not we care about rvm
        // TODO: Should we follow symlinks or what?
        let pwd = env::var("PWD")?;

        // Yeah, let's bail if we can't find our way home
        let home = env::var("HOME")?;

        let rvm_path: String = env::var("rvm_path")?;
        let rvm_path = Path::new(rvm_path.as_str()).join("gems/");

        let has_gemfile = find_ancestors("Gemfile", &pwd, &home, &rvm_path).is_some();

        // Unless forced to, skip directories without a bundler Gemfile
        if args.force_show != true && has_gemfile != true {
            return Ok(vec![]);
        }

        let requested_ruby_version: Option<PathBuf> =
            find_ancestors(".ruby-version", &pwd, &home, &rvm_path);
        let requested_ruby_version: Option<String> = match requested_ruby_version {
            None => None,
            Some(ruby_version_path) => {
                let version = std::fs::read_to_string(ruby_version_path)?;
                Some(version.trim_end_matches('\n').to_string())
            }
        };

        let gem_home: String = env::var("GEM_HOME")?;
        let cur_ruby_version = gem_home.replace(rvm_path.to_str().unwrap(), "");
        let cur_ruby_version = Gemset::<semver::Version>::from_str(cur_ruby_version.as_str())
            .map_err(|_| anyhow!("couldn't parse the current ruby version"))?;

        let mut ruby_match = true;
        if let Some(requested_ruby_version) = requested_ruby_version {
            let requested_ruby_version =
                Gemset::<semver::VersionReq>::from_str(requested_ruby_version.as_str())
                    .map_err(|_| anyhow!("couldn't parse the desired ruby version"))?;

            // Our current ruby version doesn't match the requested version sooo it's likely what we want isn't installed
            if (cur_ruby_version.interp != requested_ruby_version.interp)
                || !requested_ruby_version
                    .version
                    .matches(&cur_ruby_version.version)
            {
                ruby_match = false;
            }
        }

        // Might as well make a bit of a stink if we're using jRuby or something
        let text = match cur_ruby_version.gemset {
            Some(gemset) => format!("{} (v{})", gemset, cur_ruby_version.version),
            None => format!("{}", cur_ruby_version.version),
        };

        let text = match ruby_match {
            true => text,
            false => format!("{}{}", text, theme.mismatch_symbol),
        };

        Ok(vec![Segment {
            fg: theme.fg,
            bg: theme.bg,
            separator: Separator::Thick,
            text,
            source: "Rvm",
        }])
    }
}
