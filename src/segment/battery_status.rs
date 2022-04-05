//! The `BatteryStatus` segment shows current state-of-charge and charging status, if applicable
use anyhow::anyhow;
use battery::State as BatteryState;
use serde::{Deserialize, Serialize};

use crate::ansi::Color;
use crate::segment::{Segment, ToSegment};
use crate::{ApplicationState, Separator};

pub struct BatteryStatus {}

#[derive(Default, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Args {}

/// Theme for the [`BatteryStatus`] segment
///
/// TODO: Make the low threshold configurable
/// TODO: Add a third color band
/// TODO: Encode battery health state?
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
pub struct Theme {
    /// Foreground color when the battery is ≥ 50% state-of-charge
    pub normal_fg: Color,
    /// Background color when the battery is ≥ 50% SOC
    pub normal_bg: Color,

    /// Foreground color when the battery is below 50% SOC
    pub low_fg: Color,
    /// Background color when the battery is below 50% SOC
    pub low_bg: Color,

    /// Displayed when the computer is connected to a wall charger
    pub charging_symbol: String,

    /// Displayed when the computer is not connected to a charger
    pub discharging_symbol: String,

    /// Displayed when the battery is empty
    pub empty_symbol: String,

    /// Displayed when the battery is finished charging
    pub full_symbol: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            normal_fg: Color::Numbered(7),
            normal_bg: Color::Numbered(22),

            low_fg: Color::Numbered(7),
            low_bg: Color::Numbered(197),

            // 🔌
            charging_symbol: "\u{1f50c}".into(),
            // ⚡
            discharging_symbol: "\u{26a1}".into(),
            // ❗
            empty_symbol: "\u{2757}".into(),
            // 🔋
            full_symbol: "\u{1f50b}".into(),
        }
    }
}

impl ToSegment for BatteryStatus {
    type Args = Args;
    type Theme = Theme;

    fn to_segment(
        _args: Option<Self::Args>,
        state: &ApplicationState,
    ) -> crate::Result<Vec<Segment>> {
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
                    theme.charging_symbol
                ),
                source: "BatteryStatus::Charging",
            },
            BatteryState::Discharging | BatteryState::Unknown
                if battery.state_of_charge().value < 50.0 =>
            {
                Segment {
                    fg: theme.low_fg,
                    bg: theme.low_bg,
                    separator: Separator::Thick,
                    text: format!(
                        "{:.0}%{}",
                        battery.state_of_charge().value * 100.0,
                        theme.discharging_symbol
                    ),
                    source: "BatteryStatus::Discharging/Unknown",
                }
            }
            BatteryState::Discharging | BatteryState::Unknown => Segment {
                fg: theme.normal_fg,
                bg: theme.normal_bg,
                separator: Separator::Thick,
                text: format!(
                    "{:.0}%{}",
                    battery.state_of_charge().value * 100.0,
                    theme.discharging_symbol
                ),
                source: "BatteryStatus::Discharging/Unknown",
            },
            BatteryState::Full => Segment {
                fg: theme.normal_fg,
                bg: theme.normal_bg,
                separator: Separator::Thick,
                text: format!("100%{}", theme.full_symbol),
                source: "BatteryStatus::Full",
            },
            BatteryState::Empty => Segment {
                fg: theme.low_fg,
                bg: theme.low_bg,
                separator: Separator::Thick,
                text: format!(
                    "{:.0}%{}",
                    battery.state_of_charge().value * 100.0,
                    theme.empty_symbol
                ),
                source: "BatteryStatus::Empty",
            },
            cur_state => Err(anyhow!("unknown battery state:{:?}", cur_state))?,
        };

        Ok(vec![seg])
    }
}
