mod ddc;
mod helpers;

use crate::ddc::{DDCDisplaySwitchConfig, ModeSwitch, SwitcherConfig};
use anyhow::{Error};
use crossbeam_channel::bounded;
use log::{debug, info, warn, LevelFilter};
use rusb::{Context, Device, Hotplug, HotplugBuilder, Registration, UsbContext};
use signal_hook::consts::SIGINT;
use signal_hook::iterator::Signals;
use simplelog::{ColorChoice, Config, TerminalMode};
use std::collections::HashMap;


use std::process::Command;
use std::thread;
use std::thread::sleep;
use std::time::Duration;
use clap::Parser;

use crate::helpers::{Also, IntegerFromHexString, parse_duration};


#[derive(Parser)]
struct CliOptions {
    #[arg(short, long)]
    debug: bool,
    #[arg(short = 'v', long, value_parser = u16::from_hex_string, help = "USB Vendor ID to listen for")]
    usb_vendor_id: u16,
    #[arg(short = 'p', long, value_parser = u16::from_hex_string, help = "USB Product ID to listen for")]
    usb_product_id: u16,

    #[arg(long, value_parser = parse_duration, default_value = "300", help = "How long to pause after issuing a DDC command")]
    ddc_wait_interval: Duration,

    #[arg(long, short, num_args = 1.., help = "Monitor configuration in the format <bus_id>:<device_arrive_mode>:<device_left_mode>")]
    monitor_config: Vec<String>,
}


fn main() -> Result<(), Error> {
    let cli = CliOptions::parse();

    simplelog::TermLogger::init(
        if cli.debug {
            LevelFilter::Debug
        } else {
            LevelFilter::Info
        },
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Always,
    )?;

    let monitor_config_map = cli.monitor_config.iter()
        .map(|config| {
            let parts = config.split(':').collect::<Vec<&str>>();
            assert_eq!(3, parts.len(), "Invalid monitor config: {}", config);
            (
                parts[0].also(|val| info!("Monitor ID: {}", val)).parse().expect("Invalid monitor id"),
                ModeSwitch {
                    device_arrive_mode: u16::from_hex_string(parts[1]).expect("Not a valid hex for device arrive mode"),
                    device_left_mode: u16::from_hex_string(parts[2]).expect("Not a valid hex for device left mode"),
                }
            )
        }).collect::<HashMap<i32, ModeSwitch>>();

    let switcher_config = SwitcherConfig::new(
        cli.usb_vendor_id,
        cli.usb_product_id,
        monitor_config_map,
    );

    let (signal_channel_sender, signal_channel_receiver) = bounded(10);
    let mut signals = Signals::new([SIGINT])?;
    thread::spawn(move || {
        for sig in signals.forever() {
            warn!("Received signal {:?}", sig);
            signal_channel_sender
                .send(sig)
                .expect("Failed to send signal")
        }
    });

    let usb_context = Context::new()?;

    let boxed_callback: Box<USBHotplugCallback> = Box::new(USBHotplugCallback {
        display_switch_configs: switcher_config.display_switch_configs,
        ddc_wait_interval: cli.ddc_wait_interval,
    });

    debug!(
        "Registering hotplug callback for vendor_id: {:?} product_id: {:?}",
        switcher_config.vendor_id, switcher_config.product_id
    );
    let mut hotplug_builder = HotplugBuilder::new();
    let registration: Registration<Context> = hotplug_builder
        .vendor_id(switcher_config.vendor_id)
        .product_id(switcher_config.product_id)
        .register(&usb_context, boxed_callback)?;

    loop {
        let result = usb_context.handle_events(None);
        if result.is_err() {
            warn!("Error handling events: {:?}", result);
            break;
        }
        if !signal_channel_receiver.is_empty() {
            debug!("Signal received");
            break;
        }
    }
    info!("Done");
    usb_context.unregister_callback(registration);
    Ok(())
}

fn switch_monitor_to_input_source(bus_id: u16, input_source: u16) {
    info!(
        "Switching monitor on bus {} to input source {}",
        bus_id, input_source
    );
    let result = Command::new("ddcutil")
        .arg(format!("--bus={}", bus_id))
        .arg("setvcp")
        .arg("60")
        .arg(input_source.to_string())
        .status();
    if result.is_err() {
        warn!("Error setting input source: {:?}", result);
    }
}

struct USBHotplugCallback {
    display_switch_configs: Vec<DDCDisplaySwitchConfig>,
    ddc_wait_interval: Duration,
}

impl<T: UsbContext> Hotplug<T> for USBHotplugCallback {
    fn device_arrived(&mut self, device: Device<T>) {
        info!("Device arrived: {:?}", device);
        self.display_switch_configs.iter().for_each(|config| {
            switch_monitor_to_input_source(
                config.bus_id,
                config.device_arrive_mode,
            );
            sleep(self.ddc_wait_interval);
        });
    }

    fn device_left(&mut self, device: Device<T>) {
        info!("Device left: {:?}", device);
        self.display_switch_configs.iter().for_each(|config| {
            switch_monitor_to_input_source(
                config.bus_id,
                config.device_left_mode,
            );
            sleep(self.ddc_wait_interval);
        });
    }
}
