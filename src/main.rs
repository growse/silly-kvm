use anyhow::Error;
use crossbeam_channel::bounded;
use rusb::{Context, Device, Hotplug, HotplugBuilder, Registration, UsbContext};
use signal_hook::consts::SIGINT;
use signal_hook::iterator::Signals;
use std::thread;

fn main() -> Result<(), Error> {
    let (signal_channel_sender, signal_channel_receiver) = bounded(10);
    let mut signals = Signals::new([SIGINT])?;
    thread::spawn(move || {
        for sig in signals.forever() {
            println!("Received signal {:?}", sig);
            signal_channel_sender
                .send(sig)
                .expect("Failed to send signal")
        }
    });

    let context = Context::new()?;
    let boxed_callback: Box<HotplugCallback> = Box::new(HotplugCallback {});
    let _: Registration<Context> = HotplugBuilder::new().register(&context, boxed_callback)?;

    println!("Waiting for thingie");
    loop {
        context.handle_events(None)?;
        if !signal_channel_receiver.is_empty() {
            break;
        }
    }
    println!("Done");
    Ok(())
}

struct HotplugCallback {}

impl<T: UsbContext> Hotplug<T> for HotplugCallback {
    fn device_arrived(&mut self, device: Device<T>) {
        println!("Device arrived: {:?}", device);
    }

    fn device_left(&mut self, device: Device<T>) {
        println!("Device left: {:?}", device)
    }
}
