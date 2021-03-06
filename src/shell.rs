//! Command shell identification and initialization.

use std::env;
use std::path::Path;

use anyhow::{anyhow, Result};
use indoc::indoc;

/// Initialization and identification of the command shell that's running promptr.
///
/// TODO: Add support for other common shells
#[derive(Copy, Clone, Debug)]
pub enum Shell {
    Bash,
}

impl Shell {
    /// Variables we want to capture from `bash`, some are computed and some are not exported
    ///
    /// Note: `wc` is not a builtin so we're probably better off splitting the string rust-side.
    ///
    /// Note: `dirs -p` prints each item on the stack on a separate line, sidestepping the paths with spaces issue
    const CAPTURE_VARS: &'static str =
        r#"uid="${UID}" hostname=${HOSTNAME} code=${?} dirs=$(dirs -p) jobs=$(jobs -p | wc -l)"#;

    /// Returns an [`anyhow::Result`] with the invoking shell or an error if the shell cannot be identified.
    pub fn get_current_shell() -> Result<Self> {
        let shell: String = env::var("PROMPTR_SHELL")
            .or_else::<anyhow::Error, _>(|_| {
                // pid_t is u32 in rust but POSIX defies it as a signed integer…
                #[cfg(any(target_os = "macos", target_os = "freebsd"))]
                let shell_via_parent =
                    crate::ffi::get_process_name(std::os::unix::process::parent_id() as i64);
                #[cfg(not(any(target_os = "macos", target_os = "freebsd")))]
                let shell_via_parent = "bash".to_string();

                match shell_via_parent {
                    _ if shell_via_parent.is_empty() => Err(anyhow!("Couldn't determine shell")),
                    shell => Ok(shell),
                }
            })
            .and_then(|shell| {
                let path = Path::new(shell.as_str());
                Ok(path
                    .file_name()
                    .ok_or_else(|| anyhow!("Couldn't determine shell"))?
                    .to_string_lossy()
                    .into())
            })?;

        match shell.as_str() {
            "bash" => Ok(Shell::Bash),
            other_shell => Err(anyhow!(
                "This shell is incompatible with promptr: {}",
                other_shell
            )),
        }
    }

    pub fn generate_loader(&self, self_exe: &str) {
        match self {
            Self::Bash => {
                println!(
                    indoc!(
                    r##"
                        if [[ $- == *i* ]]; then
                            promptr_conf_dir=$({promptr} location)
                            promptr_conf_file="${{promptr_conf_dir}}/promptr.json"

                            if [ ! -f "${{promptr_conf_file}}" ]; then
                                echo "Couldn't find an existing configuration file, using the defaults"
                            fi

                            unset promptr_conf_dir
                            unset promptr_conf_file

                            PROMPT_COMMAND=promptr_prompt
                            promptr_prompt() {{
                                PS1="$({capture_vars} {promptr} prompt)"
                            }}
                        fi
                    "##
                    ),
                    capture_vars = Self::CAPTURE_VARS,
                    promptr = self_exe,
                )
            }
        }
    }
}
