use std::collections::HashMap;

use crate::helpers::{Also, IntegerFromHexString};
use log::{debug, info, warn};
use std::process::Command;

pub struct ModeSwitch {
    pub device_arrive_mode: u16,
    pub device_left_mode: u16,
}

pub struct DDCDisplaySwitchConfig {
    pub display_bus_id: u16,
    pub device_arrive_mode: u16,
    pub device_left_mode: u16,
}

pub struct SwitcherConfig {
    pub vendor_id: u16,
    pub product_id: u16,
    pub display_switch_configs: Vec<DDCDisplaySwitchConfig>,
}

pub fn parse_monitor_config(config: Vec<String>) -> HashMap<i32, ModeSwitch> {
    return config
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
        .collect::<HashMap<i32, ModeSwitch>>();
}

type DisplayId = i32;

impl SwitcherConfig {
    pub fn new(
        vendor_id: u16,
        product_id: u16,
        displays_to_modes: HashMap<i32, ModeSwitch>,
    ) -> Self {
        let displays = Command::new("ddcutil")
            .arg("detect")
            .arg("--terse")
            .output()
            .expect("Unable to detect displays");

        let mut display_to_bus = HashMap::new();
        let mut current_display: Option<DisplayId> = None;
        std::str::from_utf8(&displays.stdout)
            .expect("Display detection output was not a string")
            .split('\n')
            .for_each(|line| {
                if line.trim().starts_with("Display") {
                    let display_number: DisplayId = line
                        .split(' ')
                        .last()
                        .expect("Display detection output was not a string")
                        .trim()
                        .also(|s| debug!("Display number: {}", s))
                        .parse()
                        .expect("Display number was not a number");
                    current_display = Some(display_number);
                } else if line.trim().starts_with("I2C bus") && current_display.is_some() {
                    let bus_id: u16 = line
                        .split('-')
                        .last()
                        .expect("Display bus detection output was not a string")
                        .also(|s| debug!("Bus id: {}", s))
                        .parse()
                        .expect("Bus id was not a number");
                    display_to_bus.insert(current_display.expect("No display found"), bus_id);
                }
            });

        let display_switch_configs = displays_to_modes
            .iter()
            .filter_map(|(display_id, mode_switch)| {
                if let Some(bus_id) = display_to_bus.get(display_id) {
                    Some(DDCDisplaySwitchConfig {
                        display_bus_id: bus_id.to_owned(),
                        device_arrive_mode: mode_switch.device_arrive_mode,
                        device_left_mode: mode_switch.device_left_mode,
                    })
                } else {
                    warn!("Display ID: {} Bus ID: Not found", display_id);
                    None
                }
            })
            .collect::<Vec<DDCDisplaySwitchConfig>>();

        SwitcherConfig {
            vendor_id,
            product_id,
            display_switch_configs,
        }
    }
}
