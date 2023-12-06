use rusb::{DeviceHandle, InterfaceDescriptor, UsbContext};

const HID: u8 = 0x03;

const HID_KEYBOARD: u8 = 0x01;
const HID_MOUSE: u8 = 0x02;

const KB_CTRL: u8 = 0x1 << 0;
const KB_SHIFT: u8 = 0x1 << 1;
const KB_ALT: u8 = 0x1 << 2;
const KB_GUI: u8 = 0x1 << 3;
const KB_RCTRL: u8 = 0x1 << 4;
const KB_RSHIFT: u8 = 0x1 << 5;
const KB_RALT: u8 = 0x1 << 6;
// const KB_RGUI: u8 = 0x1 << 7;

const M_LEFT: u8 = 0x1 << 0;
const M_RIGHT: u8 = 0x1 << 1;
const M_MID: u8 = 0x1 << 2;
const M_RL: u8 = 0b11;
const M_RM: u8 = 0b101;
const M_LM: u8 = 0b110;
const M_RLM: u8 = 0b111;

pub fn print_read<T: UsbContext>(handle: &DeviceHandle<T>, eaddr: u8, emax:usize, prot: u8) -> bool {
    let mut modi = String::new();
    let mut buffer = vec![0u8; emax];
    match handle.read_bulk(eaddr, &mut buffer, std::time::Duration::from_secs(5)) {
        Ok(transferred) => {
            match prot {
                HID_KEYBOARD => {
                    match buffer[0] {
                        0 => modi = "None".to_string(),
                        KB_CTRL => modi = "CTRL".to_string(),
                        KB_SHIFT => modi = "SHIFT".to_string(),
                        KB_ALT => modi = "ALT".to_string(),
                        KB_GUI => modi = "GUI (Windows Key)".to_string(),
                        KB_RCTRL => modi = "Right CTRL".to_string(),
                        KB_RSHIFT => modi = "Right SHIFT".to_string(),
                        KB_RALT => modi = "Right Alt".to_string(),
                        _ => modi = "Right GUI (Windows Key)".to_string(),
                    }
                    let new_buf = &buffer[2..7];
                        // Convert bytes to String
                    // let ascii_string = String::from_utf8_lossy(&new_buf);

                    // Print the result
                    // println!("ASCII String: {}", ascii_string);
                    println!("Read {} bytes from Keyboard:\n   Control Key: {}\n   Keypresses: {:?}", transferred, modi, new_buf);
                    return true;
                }
                HID_MOUSE => {
                    match buffer[1] {
                        0 => modi = "None".to_string(),
                        M_LEFT => {
                            modi = "Left".to_string();
                            println!("Read {} bytes from Mouse:\n   Click: {}", transferred, modi);
                        }
                        M_RIGHT => {
                            modi = "Right".to_string();
                            println!("Read {} bytes from Mouse:\n   Click: {}", transferred, modi);
                        } 
                        M_MID => {
                            modi = "Middle".to_string();
                            println!("Read {} bytes from Mouse:\n   Click: {}", transferred, modi);
                        } 
                        M_RL => {
                            modi = "Right and Left".to_string();
                            println!("Read {} bytes from Mouse:\n   Click: {}", transferred, modi);
                        } 
                        M_RM => {
                            modi = "Right and Middle".to_string();
                            println!("Read {} bytes from Mouse:\n   Click: {}", transferred, modi);
                        } 
                        M_LM => {
                            modi = "Left and Middle".to_string();
                            println!("Read {} bytes from Mouse:\n   Click: {}", transferred, modi);
                        } 
                        M_RLM => {
                            modi = "Right, Left and Middle".to_string();
                            println!("Read {} bytes from Mouse:\n   Click: {}", transferred, modi);
                        } 
                        _ => {
                            modi = "Device Specific".to_string();
                            println!("Read {} bytes from Mouse:\n   Click: {}", transferred, modi);
                        } 
                    }
                    // let new_buf = &buffer[3..6];
                    // println!("Read {} bytes from Mouse:\n   Click: {}\n   Mouse Movement: {:?}", transferred, modi, new_buf);
                    return true;
                }
                _ => return true,
            }
        }
        Err(err) => {
            // eprintln!("Error reading from USB: {}", err);
            match err {
                rusb::Error::Timeout => return true,
                rusb::Error::NoDevice => return false,
                _ => return true,
            }
        }
    }    
}

pub fn check_class(idesc: &InterfaceDescriptor) -> (bool, u8){
    let class = idesc.class_code();
    let prot = idesc.protocol_code();

    match class {
        HID => {
            match prot {
                HID_KEYBOARD => return (true, HID_KEYBOARD),
                HID_MOUSE => return (true, HID_MOUSE),
                _ => return (false,0),
            }
        }
        _ => return (false,0),
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