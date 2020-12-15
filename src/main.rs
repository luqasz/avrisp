pub mod errors;
pub mod programmer;
pub mod specs;
use programmer::*;
use std::convert::*;
use std::fs::File;
use std::io::prelude::*;

const SPECS: specs::Specs = specs::atmega::ATMEGA_32;

fn main() -> Result<(), errors::ErrorKind> {
    let port = "/dev/serial/by-id/usb-microSENSE_USB_AVR_ISP_II_FT-STK500v2_FTWAKGHJ-if00-port0"
        .to_string();
    let stk = stk500v2::STK500v2::open(&port, SPECS).unwrap();
    let mut isp: stk500v2::IspMode = stk.try_into()?;
    fuses(&mut isp)?;
    lock_bytes(&mut isp)?;
    signature(&mut isp)?;
    flash(&mut isp)?;
    eeprom(&mut isp)?;
    isp.close()?;
    return Ok(());
}

fn fuses<T: programmer::AVRFuseGet>(programmer: &mut T) -> Result<(), errors::ErrorKind> {
    println!("fuses: {}", programmer.get_fuses()?);
    Ok(())
}

fn lock_bytes<T: programmer::AVRLockByteGet>(programmer: &mut T) -> Result<(), errors::ErrorKind> {
    println!("lock byte {:#04X}", programmer.get_lock_byte()?);
    Ok(())
}

fn signature<T: programmer::MCUSignature>(programmer: &mut T) -> Result<(), errors::ErrorKind> {
    let sign = programmer.get_mcu_signature()?;
    println!(
        "MCU signature: {:#04x} {:#04x} {:#04x}",
        sign.bytes.0, sign.bytes.1, sign.bytes.2
    );
    Ok(())
}

fn eeprom<T: programmer::EEPROMRead>(programmer: &mut T) -> Result<(), errors::ErrorKind> {
    let mut eeprom: Vec<u8> = vec![0; SPECS.eeprom.size];
    programmer.read(&mut eeprom)?;
    dump(&mut eeprom, String::from("eeprom.bin"));
    Ok(())
}

fn flash<T: programmer::FlashRead>(programmer: &mut T) -> Result<(), errors::ErrorKind> {
    let mut flash: Vec<u8> = vec![0; SPECS.flash.size];
    programmer.read(&mut flash)?;
    truncate(&mut flash);
    dump(&mut flash, String::from("flash.bin"));
    Ok(())
}

fn truncate(bytes: &mut Vec<u8>) {
    let found = bytes.iter().rposition(|&x| x != 0xff);
    let end = match found {
        Some(0) => 0,
        None => return,
        Some(x) => x + 1,
    };
    bytes.truncate(end);
}

fn dump(bytes: &Vec<u8>, name: String) {
    let mut file = File::create(name).unwrap();
    file.write_all(&bytes).unwrap();
}
