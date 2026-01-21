use std::collections::HashMap;

use crate::helpers::{Also, IntegerFromHexString};
use log::{debug, info};
use std::process::Command;

pub struct ModeSwitch {
    pub device_arrive_mode: u16,
    pub device_left_mode: u16,
}

pub struct DDCDisplaySwitchConfig {
    pub display_number: DisplayId,
    pub device_arrive_mode: u16,
    pub device_left_mode: u16,
}

pub struct SwitcherConfig {
    pub vendor_id: u16,
    pub product_id: u16,
    pub display_switch_configs: Vec<DDCDisplaySwitchConfig>,
}

pub fn parse_monitor_config(config: Vec<String>) -> HashMap<DisplayId, ModeSwitch> {
    config
        .iter()
        .map(|config| {
            let parts = config.split(':').collect::<Vec<&str>>();
            assert_eq!(3, parts.len(), "Invalid monitor config: {}", config);
            (
                parts[0]
                    .also(|val| info!("Monitor ID: {}", val))
                    .parse()
                    .expect("Invalid monitor id"),
                ModeSwitch {
                    device_arrive_mode: u16::from_hex_string(parts[1])
                        .expect("Not a valid hex for device arrive mode"),
                    device_left_mode: u16::from_hex_string(parts[2])
                        .expect("Not a valid hex for device left mode"),
                },
            )
        })
        .collect::<HashMap<DisplayId, ModeSwitch>>()
}

type DisplayId = u16;

impl SwitcherConfig {
    pub fn new(
        vendor_id: u16,
        product_id: u16,
        displays_to_modes: HashMap<DisplayId, ModeSwitch>,
    ) -> Self {
        let displays = Command::new("ddcutil")
            .arg("detect")
            .arg("--terse")
            .output()
            .expect("Unable to detect displays");
        if !displays.status.success() {
            panic!("Display detection failed with status: {}", displays.status);
        }
        let displays: Vec<DisplayId> = std::str::from_utf8(&displays.stdout)
            .expect("Display detection output was not a string")
            .split('\n')
            .filter_map(|line| {
                if line.trim().starts_with("Display") {
                    let display_number: DisplayId = line
                        .split(' ')
                        .next_back()
                        .expect("Display detection output was not a string")
                        .trim()
                        .also(|s| debug!("Display number: {}", s))
                        .parse()
                        .expect("Display number was not a number");
                    Some(display_number)
                } else {
                    None
                }
            })
            .collect();

        let display_switch_configs = displays_to_modes
            .iter()
            .map(|(display_id, mode_switch)| {
                if !displays.contains(display_id) {
                    panic!("Display ID {} not found in ddcutil output", display_id);
                }
                DDCDisplaySwitchConfig {
                    display_number: *display_id,
                    device_arrive_mode: mode_switch.device_arrive_mode,
                    device_left_mode: mode_switch.device_left_mode,
                }
            })
            .collect::<Vec<DDCDisplaySwitchConfig>>();
        if display_switch_configs.is_empty() {
            panic!("No displays found");
        }

        SwitcherConfig {
            vendor_id,
            product_id,
            display_switch_configs,
        }
    }
}
