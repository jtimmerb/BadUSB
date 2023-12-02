pub trait UsbCallback: Send {
    fn device_added(&mut self, dev_vid: u16, dev_pid: u16);
    fn device_removed(&mut self, dev_vid: u16, dev_pid: u16);
}