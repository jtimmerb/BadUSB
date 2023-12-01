use rusb::{InterfaceDescriptor, EndpointDescriptor};

const AUDIO: u8 = 0x01;
const COMM: u8 = 0x02;
const HID: u8 = 0x03;
const HID_KEYBOARD: u8 = 0x01;
const HID_MOUSE: u8 = 0x02;

fn print_interface(ifacedesc: &InterfaceDescriptor) {
    let class = ifacedesc.class_code();
    let sclass = ifacedesc.sub_class_code();
    let prot = ifacedesc.protocol_code();

    let (class_str, prot_str) = match class {
        AUDIO => ("Audio", "Don't Care"),
        COMM => ("Communications", "Don't Care"),
        HID => {
            let prot_str = match prot {
                HID_KEYBOARD => "Keyboard",
                HID_MOUSE => "Mouse",
                _ => "Don't Care",
            };
            ("Human Interface Device", prot_str)
        }
        _ => ("Don't Care", "Don't Care"),
    };

    println!("  Interface {}", ifacedesc.interface_number());
    println!("   Class: {} {}", class, class_str);
    println!("   Subclass: {}", sclass);
    println!("   Protocol: {} {}", prot, prot_str);
}

fn print_endpoint(endpdesc: &EndpointDescriptor) {
    let addr = endpdesc.address();
    let max_packet_size = endpdesc.max_packet_size();

    println!("    EndPoint Address: {}", addr);
    println!("    Max Packet Length: {}", max_packet_size);
}