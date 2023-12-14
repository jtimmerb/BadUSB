use usb_gadget;
use wms::{Attack, InputAttack, SnoopAttack, WMSError, WMSKeyboardDevice};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    usb_gadget::remove_all().expect("Could not remove existing USB gadget");

    let mut kybd = WMSKeyboardDevice::new();
    let reg_gadget = kybd.setup_gadget()?;
    kybd.read_script("scripts/sample-script.txt");
    kybd.input_attack()?;

    Ok(())
}
