use crate::usb;
use crate::detector;
use anyhow::{Result};

pub struct App{
    vid: u16,
    pid: u16,
}

impl usb::UsbCallback for App {
    fn device_added(&mut self, dev_vid: u16, dev_pid: u16){
        self.vid = dev_vid;
        self.pid = dev_pid;
    }

    fn device_removed(&mut self, dev_vid: u16, dev_pid: u16){
        self.vid = 0;
        self.pid = 0;
    }
}

impl App {
    pub fn new() -> Result<Self> {
        let vid = 0;
        let pid = 0;
        Ok(Self {vid, pid})
    }

    pub fn run(self) -> Result<()> {
        let detector = detector::Detect::new(Box::new(self));
        detector.detect()?;

        Ok(())
    }
}