//! The `Hostname` segment diplays the system hostname

use std::env;

use serde::{Deserialize, Serialize};
use sysctl::{Ctl, Sysctl};

use crate::segment::{Segment, ToSegment};
use crate::{ApplicationState, Color, Separator};

pub struct Hostname {}

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Args {
    pub hide_domain: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Theme {
    pub fg: Color,
    pub bg: Color,
}

impl Default for Args {
    fn default() -> Self {
        Self { hide_domain: true }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            fg: Color::Numbered(250),
            bg: Color::Numbered(238),
        }
    }
}

impl ToSegment for Hostname {
    type Args = Args;
    type Theme = Theme;

    fn to_segment(args: Option<Self::Args>, state: &ApplicationState) -> crate::Result<Vec<Segment>> {
        let args = args.unwrap_or_default();
        let Theme { fg, bg } = state.theme.hostname;

        let hostname = env::var("hostname").expect("Hostname not set, check init");
        let hostname = match args.hide_domain {
            false => hostname,
            true => hostname
                .split('.')
                .next()
                .expect("Couldn't determine hostname")
                .to_string(),
        };

        let mut jail_suffix = "";

        if let Ok(ctl) = Ctl::new("security.jail.jailed") {
            if let Ok(sysctl::CtlValue::Int(jailed)) = ctl.value() {
                if jailed == 1 {
                    jail_suffix = "👮";
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
