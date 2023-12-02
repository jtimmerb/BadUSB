use rusb::{DeviceHandle, InterfaceDescriptor, UsbContext};

const HID: u8 = 0x03;
const HID_KEYBOARD: u8 = 0x01;

pub fn is_keyboard(idesc: &InterfaceDescriptor) -> bool{
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

pub fn detach_interface<T: UsbContext>(handle: &mut DeviceHandle<T>, iface_num: u8){
    match handle.detach_kernel_driver(iface_num){
        Ok(_) => {
            println!("Detached Interface");
        }
        Err(err) => {
            eprintln!("Error detaching USB Interface: {}", err);
        }
    }
}

pub fn claim_interface<T: UsbContext>(handle: &mut DeviceHandle<T>, iface_num: u8){
    match handle.claim_interface(iface_num){
        Ok(_) => {
            println!("Claimed Interface");
        }
        Err(err) => {
            eprintln!("Error claiming USB Interface: {}", err);
        }
    }
}