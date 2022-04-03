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
    /// Returns an [`anyhow::Result`] with the invoking shell or an error if the shell cannot be identified.
    pub fn get_current_shell() -> Result<Self> {
        let shell: String = env::var("PROMPTR_SHELL")
            .or_else::<anyhow::Error, _>(|_| {
                let shell = env::var("SHELL")?;
                Ok(shell)
            })
            .and_then(|shell| {
                let path = Path::new(shell.as_str());
                Ok(path
                    .file_name()
                    .ok_or_else(|| anyhow!("Couldn't detemine shell"))?
                    .to_string_lossy()
                    .into())
            })?;

        match shell.as_str() {
            "bash" => Ok(Shell::Bash),
            _ => Err(anyhow!("This shell is incompatible with promptr")),
        }
    }

    /// Prints out the initialization code for the selected shell.
    pub fn generate_init(&self, self_exe: &str) {
        match self {
            Self::Bash => {
                println!(
                    indoc!(
                    r##"
                        promptr_conf_dir=$({promptr} dump-location)
                        promptr_conf_file="${{promptr_conf_dir}}/promptr.json"
                        if [ ! -d "${{promptr_conf_dir}}" ]; then
                            echo "Creating default configuration directory"
                            mkdir "${{promptr_conf_dir}}"
                        fi
                        if [ ! -f "${{promptr_conf_file}}" ]; then
                            echo "Saving default configuration at ${{promptr_conf_file}}"
                            {promptr} dump-config > "${{promptr_conf_file}}"
                        else
                            echo "Found an existing configuration at ${{promptr_conf_file}}"
                        fi

                        unset promptr_conf_dir
                        unset promptr_conf_file

                        alias promptr-seg='hostname=$HOSTNAME code=123 {promptr} dump-segment'
                        alias promptr-config='{promptr} dump-config'
                        alias promptr-location='{promptr} dump-location'

                        PROMPT_COMMAND=promptr_prompt
                        promptr_prompt() {{
                            PS1="$(hostname=$HOSTNAME code=$? jobs=$(jobs -p | wc -l) {promptr} prompt)"
                        }}
                    "##
                    ),
                    promptr = self_exe,
                )
            }
        }
    }
}
