use libusb::{Context, DeviceHandle};
//extern crate hidapi;

//const api = hidapi::HidApi::new().unwrap();

const AUDIO: u8 = 0x01;
const COMM: u8 = 0x02;
const HID: u8 = 0x03;
const HID_KEYBOARD: u8 = 0x01;
const HID_MOUSE: u8 = 0x02;

fn print_interface(ifacedesc: &libusb::InterfaceDescriptor) {
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

fn print_endpoint(endpdesc: &libusb::EndpointDescriptor) {
    let addr = endpdesc.address();
    let max_packet_size = endpdesc.max_packet_size();

    println!("    EndPoint Address: {}", addr);
    println!("    Max Packet Length: {}", max_packet_size);
}

fn read_from_usb(handle: &mut DeviceHandle, addr: u8, max_packet_size: u16, iface_num: u8) {
    	
    let mut buffer = vec![0u8; max_packet_size.into()];
    match handle.claim_interface(iface_num){
	Ok(_) => {
	    println!("Claimed Interface");
	}
	Err(err) => {
	    eprintln!("Error claiming USB Interface: {}", err);
	}
    }
    match handle.read_bulk(addr, &mut buffer, std::time::Duration::from_secs(1)) {
        Ok(transferred) => {
            println!("Read {} bytes from USB: {:?}", transferred, buffer);
        }
        Err(err) => {
            eprintln!("Error reading from USB: {}", err);
        }
    }
    
}

fn list_usb_interfaces() {
    let context = Context::new().expect("Failed to initialize libusb context");
    let devices = context.devices().expect("Failed to get device list");

    println!("List of USB interfaces:");

    for device in devices.iter() {
        let device_desc = device.device_descriptor().expect("Failed to get device descriptor");
        println!("Device {:04x}:{:04x}", device_desc.vendor_id(), device_desc.product_id());

        let mut handle = match device.open() {
            Ok(handle) => handle,
            Err(_) => {
                println!("Error Opening Device");
                continue;
            }
        };

        for config_num in 0..device_desc.num_configurations(){
            let config = device.config_descriptor(config_num).expect("No config descriptor");
            for iface in config.interfaces() {
                let ifacedesc = iface.descriptors().next().expect("No interface descriptors");
                print_interface(&ifacedesc);
                for endpdesc in ifacedesc.endpoint_descriptors() {
                    print_endpoint(&endpdesc);
                    read_from_usb(&mut handle, endpdesc.address(), endpdesc.max_packet_size(), ifacedesc.interface_number());
                }
            }
        }
    }
}

fn main() {
    list_usb_interfaces();
}

