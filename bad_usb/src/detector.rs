use crate::usb::{UsbCallback};
use anyhow::{Result};
use rusb::{Context, Device, HotplugBuilder, UsbContext};

pub struct Detect {
    callback: Box<dyn UsbCallback>,
}

impl<T: UsbContext> rusb::Hotplug<T> for Detect {
    fn device_arrived(&mut self, device: Device<T>){
        let dev_desc = device.device_descriptor().expect("No descriptor");
        println!("Device Added: {:?}", device);
        self.callback.device_added(dev_desc.vendor_id(), dev_desc.product_id());
    }

    fn device_left(&mut self, device: Device<T>){
        let dev_desc = device.device_descriptor().expect("No descriptor");
        println!("Device Removed: {:?}", device);
        self.callback.device_removed(dev_desc.vendor_id(), dev_desc.product_id());
    }
}

impl Detect {
    pub fn new(callback: Box<dyn UsbCallback>) -> Self {
        Detect { callback }
    }
    
    pub fn detect(self) -> Result<()> {
        if rusb::has_hotplug() {
            let context = Context::new()?;

            let _reg = HotplugBuilder::new()
                .register::<Context, &Context>(&context, Box::new(Detect {callback: self.callback}))?;

            loop {
                if let Err(err) = context.handle_events(None) {
                    eprint!("Error during USB errors handling: {:?}", err);
                    break;
                };
            }
            Ok(())
        } else {
            // This should never happen: hotplug is supported on Linux and MacOS both.
            eprint!("libusb hotplug api unsupported");
            Ok(())
        }
    }
}