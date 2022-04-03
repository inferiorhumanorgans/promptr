//! The `BatteryStatus` segment shows current state-of-charge and charging status, if applicable
use anyhow::anyhow;
use battery::State as BatteryState;
use serde::{Deserialize, Serialize};

use crate::segment::{Segment, ToSegment};
use crate::{ApplicationState, Color, Separator};

pub struct BatteryStatus {}

#[derive(Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Args {
    charging_symbol: String,
    discharging_symbol: String,
    empty_symbol: String,
    full_symbol: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Theme {
    pub normal_fg: Color,
    pub normal_bg: Color,

    pub low_fg: Color,
    pub low_bg: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            normal_fg: Color::Numbered(7),
            normal_bg: Color::Numbered(22),

            low_fg: Color::Numbered(7),
            low_bg: Color::Numbered(197),
        }
    }
}

impl Default for Args {
    fn default() -> Self {
        Self {
            charging_symbol: "🔌".into(),
            discharging_symbol: "⚡".into(),
            empty_symbol: "❗".into(),
            full_symbol: "🔋".into(),
        }
    }
}

impl ToSegment for BatteryStatus {
    type Args = Args;
    type Theme = Theme;

    fn to_segment(args: Option<Self::Args>, state: &ApplicationState) -> crate::Result<Vec<Segment>> {
        let args = args.unwrap_or_default();

        let theme = &state.theme.battery;

        let manager = battery::Manager::new()?;
        let battery = manager.batteries()?.next().unwrap()?;

        let seg = match battery.state() {
            BatteryState::Charging => Segment {
                fg: theme.normal_fg,
                bg: theme.normal_bg,
                separator: Separator::Thick,
                text: format!(
                    "{:.0}%{}",
                    battery.state_of_charge().value * 100.0,
                    args.charging_symbol
                ),
                source: "BatteryStatus::Discharged/Unknown",
            },
            BatteryState::Discharging | BatteryState::Unknown => Segment {
                fg: theme.normal_fg,
                bg: theme.normal_bg,
                separator: Separator::Thick,
                text: format!(
                    "{:.0}%{}",
                    battery.state_of_charge().value * 100.0,
                    args.discharging_symbol
                ),
                source: "BatteryStatus::Discharged/Unknown",
            },
            BatteryState::Full => Segment {
                fg: theme.normal_fg,
                bg: theme.normal_bg,
                separator: Separator::Thick,
                text: format!("100%{}", args.full_symbol),
                source: "BatteryStatus::Full",
            },
            BatteryState::Empty => Segment {
                fg: theme.low_fg,
                bg: theme.low_bg,
                separator: Separator::Thick,
                text: format!(
                    "{:.0}%{}",
                    battery.state_of_charge().value * 100.0,
                    args.empty_symbol
                ),
                source: "BatteryStatus::Empty",
            },
            cur_state => Err(anyhow!("unknown battery state:{:?}", cur_state))?,
        };

        Ok(vec![seg])
    }
}
