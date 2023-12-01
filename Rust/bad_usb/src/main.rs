use rusb::{Context, Device, DeviceHandle, InterfaceDescriptor, UsbContext};
use anyhow::{Result};
use std::sync::{Arc, Mutex};
use std::thread;
mod app;
mod usb;
mod detector;

const HID: u8 = 0x03;
const HID_KEYBOARD: u8 = 0x01;

fn read_indef<T>(device: Device<T>)
where
    T: UsbContext,
{
    let mut handle = device.open().expect("Unable to open device");
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

fn main() -> Result<()> {
    let app = app::App::new()?;
    app.run()?;
    Ok(())
}