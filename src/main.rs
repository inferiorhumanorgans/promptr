use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand};
use directories::ProjectDirs;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use serde_json::from_reader as json_from_reader;

use std::env;
use std::fmt::{self, Display};
use std::fs::{self, File};
use std::path::PathBuf;

pub mod ansi;
use ansi::Color;

pub mod segment;
use segment::{Segment, ToSegment};

pub mod shell;
use shell::Shell;

/// Processes a segment struct and returns the relevant segment(s), if any.
#[macro_export]
macro_rules! run_segment_with_args {
    ($klass:ident, $args:ident, $state:ident) => {
        match $args {
            None => segment::$klass::to_segment(None, &$state).ok(),
            Some(args) => match serde_json::from_value(args) {
                Ok(args) => segment::$klass::to_segment(Some(args), &$state).ok(),
                Err(e) => {
                    eprintln!("{} args: {}", stringify!($klass), e);
                    None
                }
            },
        }
    };
}

/// promptr is a colorful, rusty prompt generator for bash.
#[derive(Parser)]
#[doc(hidden)]
#[clap(author, version, propagate_version = true, max_term_width = 80)]
struct TopLevelArgs {
    #[clap(subcommand)]
    command: Commands,
}

#[doc(hidden)]
#[derive(Subcommand, PartialEq)]
enum Commands {
    /// Prints out the shell commands required to load and enable
    /// promptr for the first run.
    ///
    /// From a bash instance run: source <(promptr init)
    Init,

    /// Print the current state of a segment.
    ///
    /// This command takes one argument: the index of a segment to display.  Running this command
    /// will print out a plain text representation of the foreground and background colors, text,
    /// and separator for the specified segment.
    Segment(SubCmdDumpSegmentArgs),

    /// Print the current configuration as JSON
    CurrentConfig,

    /// Print the default configuration in all its glory
    DefaultConfig,

    /// Print the location of the configuration directory
    Location,

    /// Same as init but without attempting to create/copy a default config file
    Load,

    /// This subcommand generates the prompt displayed by the command shell.  Don't call directly
    Prompt,
}

#[doc(hidden)]
#[derive(Args, Debug, PartialEq)]
struct SubCmdDumpSegmentArgs {
    idx: usize,
}

/// Separator shown between segments
///
/// Typically the thick separator is used unless the background of two adjacent segments is the same.
#[derive(Debug)]
pub enum Separator {
    Thin,
    Thick,
}

/// This represents a stanza in the config file that describes a sgement. The `args` field is typed
/// specifically for each segment, and each segment implements `serde(default)` so you only need to
/// specify the fields you wish to override.
#[derive(Deserialize, Debug, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SegmentConfig {
    name: String,
    #[serde(skip_serializing_if = "SegmentConfig::serialize_optional_json")]
    args: Option<serde_json::Value>,
}

/// Represents the contents of a JSON config file.
///
/// The available segments are described in the [`segment`] module.  If no config file is found,
/// the defaults are used.  Both the active and default configurations can be viewed in JSON via
/// the `promptr current-config` and `promptr default-config` commands respectively.
#[derive(Deserialize, Debug, Serialize)]
pub struct PromptrConfig {
    /// Magic number, currently needs to be 12.
    pub promptr_config: u32,

    /// List of segments to render for the left prompt.
    pub segments: Vec<SegmentConfig>,

    /// Theme options.  Each module under [`segment`] defines a Theme object with the configurable
    /// colors specific to each segment.  The only parts that need to be specified are those that
    /// you wish to override.  For instance to override only the background color for the [`Hostname`](`segment::hostname`)
    /// segment your theme stanza would look like this:
    /// ```json
    /// {
    ///     "hostname": { "bg": 128 }
    /// }
    /// ```
    ///
    /// In this case `bg` is a [`Color`] object which can be represented by an integer.
    #[serde(default)]
    pub theme: Theme,
}

/// Global application state.  Includes information that we've captured from the shell and theme
/// information.
#[derive(Debug)]
pub struct ApplicationState<'a> {
    exit_code: u8,
    theme: &'a Theme,
}

/// Contains colors for the active theme.
///
/// All fields implement `serde(default)` and are thus optional.
#[derive(Clone, Default, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Theme {
    /// Theme for the [`battery_status`](`segment::battery_status`) segment.
    #[cfg(feature = "segment-battery")]
    pub battery: segment::battery_status::Theme,

    /// Theme for the [`command_status`](`segment::command_status`) segment.
    pub command_status: segment::command_status::Theme,

    /// Theme for the [`hostname`](`segment::hostname`) segment.
    pub hostname: segment::hostname::Theme,

    /// Theme for the version control segments including the [`git`](`segment::git`) segment.
    #[cfg(feature = "segment-git")]
    pub vcs: segment::vcs::Theme,

    pub username: segment::username::Theme,

    pub paths: segment::paths::Theme,
}

impl Display for Separator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Thin => write!(f, "\u{e0b1}"),
            Self::Thick => write!(f, "\u{e0b0}"),
        }
    }
}

impl SegmentConfig {
    /// We can end up with Some(Null) instead of None sometimes because reasons.
    /// This ensure serde skips writing those out.
    fn serialize_optional_json(value: &Option<serde_json::Value>) -> bool {
        match value {
            Some(value) => {
                if matches!(value, serde_json::Value::Null) {
                    true
                } else {
                    false
                }
            }
            None => true,
        }
    }
}

impl Default for PromptrConfig {
    /// Default segments to render.
    fn default() -> Self {
        Self {
            promptr_config: 12,
            segments: vec![
                SegmentConfig {
                    name: "username".into(),
                    args: None,
                },
                SegmentConfig {
                    name: "paths".into(),
                    args: None,
                },
                SegmentConfig {
                    name: "command_status".into(),
                    args: None,
                },
            ],

            theme: Theme::default(),
        }
    }
}

#[doc(hidden)]
fn config_dir() -> Result<PathBuf> {
    let project_dirs = ProjectDirs::from("com", "inferiorhumanorgans", "promptr")
        .ok_or_else(|| anyhow!("couldn't create ProjectDirs"))?;
    let config_dir = project_dirs.config_dir();

    if let Err(error) = fs::metadata(config_dir) {
        match error.kind() {
            std::io::ErrorKind::NotFound => {
                fs::create_dir_all(config_dir)?;
            }
            _ => Err(error)?,
        }
    };

    Ok(config_dir.into())
}

/// Loads the configuration from disk
///
/// ## Arguments
///
/// * `quiet` – Whether or not to print parsing errors to STDERR
pub fn load_config(quiet: bool) -> PromptrConfig {
    let config_file_path: String = match config_dir() {
        Ok(config_dir) => {
            let path = config_dir.join("promptr.json");
            path.into_os_string().to_string_lossy().into()
        }
        Err(_) => "".into(),
    };

    File::open(config_file_path)
        .map_err(|e| e.into()) // Into anyhow
        .and_then(|file| {
            json_from_reader(file).map_err(|e| {
                if !quiet {
                    eprintln!("JSON parsing error, using default config.");
                    eprintln!("{:?}", e);
                }
                anyhow!("{}", e)
            })
        })
        .unwrap_or_default()
}

/// Runs through the current configuration and renders each segment.
///
/// ## Arguments
///
/// * `config` – the configuration instance to iterate over
///
/// ## Returns
///
/// An iterator over [`Segment`].
pub fn load_segments(config: PromptrConfig) -> Result<impl Iterator<Item = Segment>> {
    let exit_code: u8 = env::var("code")?.parse::<u8>()?;

    let state = ApplicationState {
        exit_code,
        theme: &config.theme,
    };

    assert_eq!(config.promptr_config, 12);

    let segments = config
        .segments
        .into_iter()
        .filter_map(|SegmentConfig { name, args }| match name.as_str() {
            "username" => run_segment_with_args!(Username, args, state),
            "hostname" => run_segment_with_args!(Hostname, args, state),
            "paths" => run_segment_with_args!(Paths, args, state),
            "command_status" => run_segment_with_args!(CommandStatus, args, state),

            #[cfg(feature = "segment-battery")]
            "battery" => run_segment_with_args!(BatteryStatus, args, state),

            #[cfg(feature = "segment-git")]
            "git" => run_segment_with_args!(Git, args, state),

            seg => {
                eprintln!("Unknown segment: {}", seg);
                None
            }
        })
        .collect_vec()
        .into_iter()
        .flatten();

    Ok(segments)
}

#[doc(hidden)]
fn main() -> Result<()> {
    let args = TopLevelArgs::parse();

    let self_exe: String = env::current_exe()?.to_string_lossy().into();

    let shell = Shell::get_current_shell()?;

    match args.command {
        Commands::Init => shell.generate_init(&self_exe),
        Commands::Load => shell.generate_loader(&self_exe),
        Commands::Prompt => {
            let config = load_config(false);
            let segments = load_segments(config)?;

            let mut it = segments.into_iter().peekable();

            while let Some(seg) = it.next() {
                let separator_bg = if let Some(next_seg) = it.peek() {
                    next_seg.bg.set_bg()
                } else {
                    ansi::Color::reset_colors()
                };

                print!(
                    "{}{} {} {}{}{}",
                    seg.fg.set_fg(),
                    seg.bg.set_bg(),
                    seg.text,
                    separator_bg,
                    seg.bg.set_fg(),
                    seg.separator
                );
            }

            print!("{} ", ansi::Color::reset_colors());
        },
        Commands::Segment(args) => {
            let config = load_config(false);

            // Mock the variables needed to render the segments
            env::set_var("code", "123");
            env::set_var("hostname", "dummy-hostname.dummy-domain");

            let segments = load_segments(config)?.collect_vec();

            match segments.get(args.idx) {
                Some(seg) => eprintln!("{:#?}", seg),
                None => eprintln!("Segment not found, count={}", segments.len()),
            }
        }
        Commands::DefaultConfig => {
            let config = PromptrConfig::default();
            println!("{}", serde_json::to_string_pretty(&config).unwrap());
        },
        Commands::CurrentConfig => {
            let config = load_config(true);

            println!("{}", serde_json::to_string_pretty(&config).unwrap());
        }
        Commands::Location => match config_dir() {
            Ok(dir) => println!("{}", dir.to_str().unwrap()),
            Err(_) => eprintln!("I couldn't find a good place to keep my configuration files."),
        },
    }

    Ok(())
}
