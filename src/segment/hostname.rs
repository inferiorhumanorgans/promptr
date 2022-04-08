//! The `Hostname` segment diplays the system hostname

use anyhow::anyhow;
use serde::{Deserialize, Serialize};

#[cfg(target_os = "freebsd")]
use sysctl::{Ctl, Sysctl};

use crate::ansi::Color;
use crate::segment::{Segment, ToSegment};
use crate::{ApplicationState, Separator};

pub struct Hostname {}

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Args {
    pub show_domain: bool,
    pub show_jail_indicator: bool,
    pub show_os_indicator: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Theme {
    /// Foreground color
    pub fg: Color,

    /// Background color
    pub bg: Color,

    /// Indicator to append if we're in a FreeBSD jail
    pub jail_indicator: String,

    /// Indicator to append if we're running on macOS
    pub os_macos: String,

    /// Indicator to append if we're running on FreeBSD
    pub os_freebsd: String,

    /// Indicator to append if we're running on OpenBSD
    pub os_openbsd: String,

    /// Indicator to append if we're running on Linux
    pub os_linux: String,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            show_domain: false,
            show_jail_indicator: true,
            show_os_indicator: false,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            fg: Color::Numbered(250),
            bg: Color::Numbered(238),

            // 🔐 – lock and key
            jail_indicator: "\u{1f510}".into(),

            // 🍎 – duh
            os_macos: "\u{1f34e}".into(),
            // 👺 – beastie
            os_freebsd: "\u{1f47a}".into(),
            // 🐡 - puffy
            os_openbsd: "\u{1f421}".into(),
            // 🐧 – tux
            os_linux: "\u{1f427}".into(),
        }
    }
}

impl ToSegment for Hostname {
    type Args = Args;
    type Theme = Theme;

    fn error_context() -> &'static str {
        "segment::Hostname"
    }

    fn to_segment(
        args: Option<Self::Args>,
        state: &ApplicationState,
    ) -> crate::Result<Vec<Segment>> {
        let args = args.unwrap_or_default();

        let Theme { fg, bg, .. } = state.theme.hostname;
        let theme = &state.theme.hostname;

        let hostname = state
            .env
            .get("hostname")
            .ok_or_else(|| anyhow!("Hostname not set, check init"))?
            .to_string();

        let hostname = match args.show_domain {
            true => hostname,
            false => hostname
                .split('.')
                .next()
                .ok_or_else(|| anyhow!("Couldn't determine hostname"))?
                .to_string(),
        };

        let mut hostname = vec![hostname];

        if args.show_os_indicator {
            if cfg!(target_os = "macos") {
                hostname.push(theme.os_macos.to_string());
            } else if cfg!(target_os = "freebsd") {
                hostname.push(theme.os_freebsd.to_string());
            } else if cfg!(target_os = "openbsd") {
                hostname.push(theme.os_openbsd.to_string());
            } else if cfg!(target_os = "linux") {
                hostname.push(theme.os_linux.to_string());
            }
        }

        #[cfg(target_os = "freebsd")]
        if args.show_jail_indicator {
            if let Ok(ctl) = Ctl::new("security.jail.jailed") {
                if let Ok(sysctl::CtlValue::Int(jailed)) = ctl.value() {
                    if jailed == 1 {
                        hostname.push(theme.jail_indicator.to_string());
                    }
                }
            }
        }

        Ok(vec![Segment {
            bg,
            fg,
            separator: Separator::Thick,
            text: hostname.join(""),
            source: "Hostname",
        }])
    }
}
