use rusb::{Context, Device, DeviceHandle, HotplugBuilder, InterfaceDescriptor, UsbContext};
use std::{thread, time};

const HID: u8 = 0x03;
const HID_KEYBOARD: u8 = 0x01;

struct HotPlugHandler;

impl<'a, T> rusb::Hotplug<T> for HotPlugHandler
where
    T: UsbContext + 'static,
{
    fn device_arrived(&mut self, device: Device<T>) {
        let dev_desc = device.device_descriptor().expect("Error retrieving device descriptor");
        if dev_desc.vendor_id() != 0x1d6b {
            println!("device arrived {:?}", device);
            thread::spawn(|| {
                read_indef(device);
            });
        }
    }
    fn device_left(&mut self, device: Device<T>) {
        println!("device left {:?}", device);
    }
}

impl Drop for HotPlugHandler {
    fn drop(&mut self) {
        println!("HotPlugHandler dropped");
    }
}

fn read_indef<T>(device: Device<T>)
where
    T: UsbContext,
{
    let mut handle = match device.open() {
        Ok(handle) => handle,
        Err(e) => eprintln!("Device found but failed to open: {}", e),
    };
    let active_config = handle.active_configuration().expect("Failed to get active configuration");
    let config = device.config_descriptor(active_config-1).expect("No config descriptor");

    for iface in config.interfaces(){

        let idesc = iface.descriptors().next().expect("No interface descriptors");

        if is_keyboard(&idesc){
            let endpdesc = idesc.endpoint_descriptors().next().expect("No endpoint descriptors");
            detach_interface(&mut handle, idesc.interface_number());
            claim_interface(&mut handle, idesc.interface_number());
            // handle.set_auto_detach_kernel_driver(true).expect("Could not set auto detach");
            read_from_usb(&mut handle, endpdesc.address(), endpdesc.max_packet_size());
            break;
        }       
    }
}

fn is_keyboard(idesc: &InterfaceDescriptor) -> bool{
    let class = idesc.class_code();
    let prot = idesc.protocol_code();

    match class {
        HID => {
            match prot {
                HID_KEYBOARD => return true,
                _ => return false,
            }
        }
        _ => return false,
    }
}

fn detach_interface<T: UsbContext>(handle: &mut DeviceHandle<T>, iface_num: u8){
    match handle.detach_kernel_driver(iface_num){
        Ok(_) => {
            println!("Detached Interface");
        }
        Err(err) => {
            eprintln!("Error detaching USB Interface: {}", err);
        }
    }
}

fn claim_interface<T: UsbContext>(handle: &mut DeviceHandle<T>, iface_num: u8){
    match handle.claim_interface(iface_num){
        Ok(_) => {
            println!("Claimed Interface");
        }
        Err(err) => {
            eprintln!("Error claiming USB Interface: {}", err);
        }
    }
}

fn read_from_usb<T: UsbContext>(handle: &mut DeviceHandle<T>, addr: u8, max_packet_size: u16) {
    let mut buffer = vec![0u8; max_packet_size.into()];
    loop{
        match handle.read_bulk(addr, &mut buffer, std::time::Duration::from_secs(5)) {
            Ok(transferred) => {
                println!("Read {} bytes from USB: {:?}", transferred, buffer);
            }
            Err(err) => {
                eprintln!("Error reading from USB: {}", err);
                match err {
                    rusb::Error::Timeout => continue,
                    _ => {
                        break;
                    },
                }
            }
        }
    }   
}

fn run_hotplug() -> rusb::Result<()> {
    if rusb::has_hotplug() {
        let context = Context::new()?;

        let mut reg = Some(
            HotplugBuilder::new()
                .enumerate(true)
                .register(&context, Box::new(HotPlugHandler {}))?,
        );

        // loop{
        context.handle_events(None).unwrap();
        if let Some(reg) = reg.take() {
            context.unregister_callback(reg);
        }
            // break;
        // }
            

        Ok(())
    } else {
        eprint!("libusb hotplug api unsupported");
        Ok(())
    }
}

fn main() {
    let _ = run_hotplug();
    loop{}
}

