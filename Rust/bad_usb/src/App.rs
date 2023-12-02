use crate::usb;
use crate::detector;
use crate::helper::{is_keyboard, detach_interface, claim_interface};
use anyhow::{Result};
use std::sync::mpsc::{Sender, channel, TryRecvError};
use std::sync::{Arc, Mutex};
use rusb::{open_device_with_vid_pid};
use std::thread;

pub struct App{
    vid: u16,
    pid: u16,
    tx: Option<Arc<Mutex<Sender<Result<u8,TryRecvError>>>>>,
    init: u8,
}

impl usb::UsbCallback for App {
    fn device_added(&mut self, dev_vid: u16, dev_pid: u16){
        self.vid = dev_vid;
        self.pid = dev_pid;
        let (tx, rx) = channel();
        // self.tx = Some(tx.clone());
        let arc_tx = Arc::new(Mutex::new(tx));
        self.tx = Some(Arc::clone(&arc_tx));
        // let _ = self.tx.clone().expect("No transmit channel").send(0);
        self.init = 1;
        thread::spawn(move|| {
            let mut handle = open_device_with_vid_pid(dev_vid, dev_pid).unwrap();
            let device = handle.device();
            let active_config = handle.active_configuration().expect("Failed to get active configuration");
            let config = device.config_descriptor(active_config-1).expect("No config descriptor");

            for iface in config.interfaces(){

                let idesc = iface.descriptors().next().expect("No interface descriptors");

                if is_keyboard(&idesc){
                    let endpdesc = idesc.endpoint_descriptors().next().expect("No endpoint descriptors");
                    detach_interface(&mut handle, idesc.interface_number());
                    claim_interface(&mut handle, idesc.interface_number());
                    // handle.set_auto_detach_kernel_driver(true).expect("Could not set auto detach");
                    let mut buffer = vec![0u8; endpdesc.max_packet_size().into()];
                    loop{
                        match handle.read_bulk(endpdesc.address(), &mut buffer, std::time::Duration::from_secs(5)) {
                            Ok(transferred) => {
                                println!("Read {} bytes from USB: {:?}", transferred, buffer);
                            }
                            Err(err) => {
                                eprintln!("Error reading from USB: {}", err);
                                match err {
                                    rusb::Error::Timeout => continue,
                                    rusb::Error::NoDevice => break,
                                    _ => {
                                        continue;
                                    },
                                }
                            }
                        }
                        let cont = rx.try_recv();
                        //cont is received in wrong data type
                        //disconnect does not terminate thread
                        //above rusb::Error::NoDevice is what ends read thread and returns control to handler
                        match cont {
                            Ok(Ok(1)) | Err(TryRecvError::Disconnected)=> {
                                println!("Terminating.");
                                break;
                            }
                            Err(TryRecvError::Empty) => {}
                            Ok(_) => break,
                        }
                    }   
                    break;
                }       
            }
        });
    }

    fn device_removed(&mut self, dev_vid: u16, dev_pid: u16){
        self.vid = 0;
        self.pid = 0;
        if let Some(arc_tx) = &self.tx {
            let _ = arc_tx.lock().unwrap().send(Ok(1));
        }
    }
}

impl App {
    pub fn new() -> Result<Self> {
        let vid = 0;
        let pid = 0;
        let tx = None;
        let init = 0;
        Ok(Self {vid, pid, tx, init})
    }

    pub fn run(self) -> Result<()> {
        let detector = detector::Detect::new(Box::new(self));
        detector.detect()?;
        Ok(())
    }
}