use anyhow::Error;
use crossbeam_channel::bounded;
use log::{debug, info, warn, LevelFilter};
use rusb::{Context, Device, Hotplug, HotplugBuilder, Registration, UsbContext};
use signal_hook::consts::SIGINT;
use std::process::Command;
use signal_hook::iterator::Signals;
use simplelog::{ColorChoice, Config, TerminalMode};
use std::thread;

struct SwitcherConfig {
    vendor_id: u16,
    product_id: u16,
    device_arrived_display_switch_config: Vec<DDCDisplaySource>,
    device_left_display_switch_config: Vec<DDCDisplaySource>,
}


fn main() -> Result<(), Error> {
    simplelog::TermLogger::init(
        LevelFilter::Debug,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Always,
    )?;

    let switcher_config = SwitcherConfig {
        vendor_id: 0x413c,
        product_id: 0x2110,
        device_arrived_display_switch_config: vec![
            DDCDisplaySource { display_id: 1, input_source: 0x11 },
            DDCDisplaySource { display_id: 2, input_source: 0x11 },
            DDCDisplaySource { display_id: 3, input_source: 0x10 },
        ],
        device_left_display_switch_config: vec![
            DDCDisplaySource { display_id: 1, input_source: 0x10 },
            DDCDisplaySource { display_id: 2, input_source: 0x10 },
            DDCDisplaySource { display_id: 3, input_source: 0x11 },
        ],
    };

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

    let context = Context::new()?;

    let boxed_callback: Box<HotplugCallback> = Box::new(HotplugCallback {
        arrived_display_switch_config: switcher_config.device_arrived_display_switch_config,
        left_display_switch_config: switcher_config.device_left_display_switch_config,
    });

    debug!("Registering hotplug callback for vendor_id: {:?} product_id: {:?}", switcher_config.vendor_id, switcher_config.product_id);
    let mut hotplug_builder = HotplugBuilder::new();
    let registration: Registration<Context> = hotplug_builder
        .vendor_id(switcher_config.vendor_id)
        .product_id(switcher_config.product_id)
        .register(&context, boxed_callback)?;

    loop {
        let result = context.handle_events(None);
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
    context.unregister_callback(registration);
    Ok(())
}

struct DDCDisplaySource {
    display_id: i32,
    input_source: i32,
}

impl DDCDisplaySource {
    fn switch_monitor_to_input_source(&self) {
        info!("Switching monitor {} to input source {}", self.display_id, self.input_source);
        let result = Command::new("ddcutil")
            .arg(format!("--display={}", self.display_id))
            .arg("setvcp")
            .arg("60")
            .arg(self.input_source.to_string())
            .status();
        if result.is_err() {
            warn!("Error setting input source: {:?}", result);
        }
    }
}

struct HotplugCallback {
    arrived_display_switch_config: Vec<DDCDisplaySource>,
    left_display_switch_config: Vec<DDCDisplaySource>,
}

impl<T: UsbContext> Hotplug<T> for HotplugCallback {
    fn device_arrived(&mut self, device: Device<T>) {
        info!("Device arrived: {:?}", device);
        self.arrived_display_switch_config.iter().for_each(|config| config.switch_monitor_to_input_source());
    }


    fn device_left(&mut self, device: Device<T>) {
        info!("Device left: {:?}", device);
        self.left_display_switch_config.iter().for_each(|config| config.switch_monitor_to_input_source());
    }
}
