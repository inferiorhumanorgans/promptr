//! The `Hostname` segment diplays the system hostname

use std::env;

use anyhow::anyhow;
use serde::{Deserialize, Serialize};
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
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Theme {
    pub fg: Color,
    pub bg: Color,
    pub jail_indicator: String,
}

impl Default for Args {
    fn default() -> Self {
        Self { 
            show_domain: false,
            show_jail_indicator: false,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            fg: Color::Numbered(250),
            bg: Color::Numbered(238),
            jail_indicator: "🔐".into(),
        }
    }
}

impl ToSegment for Hostname {
    type Args = Args;
    type Theme = Theme;

    fn to_segment(
        args: Option<Self::Args>,
        state: &ApplicationState,
    ) -> crate::Result<Vec<Segment>> {
        let args = args.unwrap_or_default();

        let Theme { fg, bg, .. } = state.theme.hostname;

        let hostname = env::var("hostname").map_err(|_| anyhow!("Hostname not set, check init"))?;
        let hostname = match args.show_domain {
            true => hostname,
            false => hostname
                .split('.')
                .next()
                .expect("Couldn't determine hostname")
                .to_string(),
        };

        let mut jail_suffix = String::new();

        #[cfg(target_family = "unix")]
        if let Ok(ctl) = Ctl::new("security.jail.jailed") {
            if let Ok(sysctl::CtlValue::Int(jailed)) = ctl.value() {
                if jailed == 1 {
                    jail_suffix = state.theme.hostname.jail_indicator.clone();
                }
            }
        }

        Ok(vec![
            Segment {
                bg,
                fg,
                separator: Separator::Thick,
                text: format!("{}{}", hostname, jail_suffix),
                source: "Hostname",
            }
        ])
    }
}
