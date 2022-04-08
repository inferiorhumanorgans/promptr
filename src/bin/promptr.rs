use anyhow::{anyhow, Result};
use clap::{Args, Parser, Subcommand};
use directories::ProjectDirs;
use itertools::Itertools;
use serde_json::from_reader as json_from_reader;

use std::collections::HashMap;
use std::env;
use std::fs::{self, File};
use std::path::PathBuf;

use libpromptr::ansi::Color;
use libpromptr::segment::{self, Segment, ToSegment};
use libpromptr::shell::Shell;
use libpromptr::{ApplicationState, PromptrConfig, SegmentConfig, Separator};

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
    let state = ApplicationState {
        theme: &config.theme,
        env: env::vars().fold(HashMap::new(), |mut acc, (key, value)| {
            acc.insert(key, value);
            acc
        }),
    };

    assert_eq!(config.promptr_config, 12);

    let segments = config
        .segments
        .into_iter()
        .map(|SegmentConfig { name, args }| match name.as_str() {
            "command_status" => segment::CommandStatus::to_segment_generic(args, &state),
            "hostname" => segment::Hostname::to_segment_generic(args, &state),
            "paths" => segment::Paths::to_segment_generic(args, &state),
            "rvm" => segment::Rvm::to_segment_generic(args, &state),
            "screen" => segment::Screen::to_segment_generic(args, &state),
            "username" => segment::Username::to_segment_generic(args, &state),

            #[cfg(feature = "segment-battery")]
            "battery" => segment::BatteryStatus::to_segment_generic(args, &state),

            #[cfg(feature = "segment-git")]
            "git" => segment::Git::to_segment_generic(args, &state),

            seg => {
                eprintln!("Unknown segment: {}", seg);
                Err(anyhow!("Unknown segment"))
            }
        })
        .filter_map(|segment_result| match segment_result {
            Ok(unflat_segments) => Some(unflat_segments),
            Err(err) => {
                eprintln!("Error: {:?}", err);
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
            let thin_separator_fg = config.theme.thin_separator_fg;
            let segments = load_segments(config)?;

            let mut it = segments.into_iter().peekable();

            while let Some(seg) = it.next() {
                let mut separator = seg.separator;
                if let Some(next_seg) = it.peek() {
                    if seg.bg == next_seg.bg {
                        separator = Separator::Thin;
                    }
                }

                let separator_fg = match separator {
                    Separator::Thick => seg.bg.set_fg(),
                    Separator::Thin => thin_separator_fg.set_fg(),
                };

                let separator_bg = if let Some(next_seg) = it.peek() {
                    next_seg.bg.set_bg()
                } else {
                    Color::reset_colors()
                };

                print!(
                    "{}{} {} {}{}{}",
                    seg.fg.set_fg(),
                    seg.bg.set_bg(),
                    seg.text,
                    separator_bg,
                    separator_fg,
                    separator
                );
            }

            print!("{} ", Color::reset_colors());
        }
        Commands::Segment(args) => {
            let config = load_config(false);

            // Mock the variables needed to render the segments
            // It's worth thinking about moving this back into a bash alias
            // so we get these variables along the same code path.
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
        }
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
