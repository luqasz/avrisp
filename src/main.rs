pub mod errors;
pub mod programmer;
use avrisp;
use programmer::stk500v2;
use programmer::stk500v2::IspMode;
use programmer::{AVRFuseGet, AVRLockByteGet, MCUSignature, Programmer};
use std::convert::TryInto;
use std::fs::File;
use std::io::prelude::*;

const SPECS: avrisp::Specs = avrisp::atmega::ATMEGA_32;

fn main() -> Result<(), errors::ErrorKind> {
    let mut flash: Vec<u8> = vec![0; SPECS.flash.size];
    let mut eeprom: Vec<u8> = vec![0; SPECS.eeprom.size];
    let port = "/dev/serial/by-id/usb-microSENSE_USB_AVR_ISP_II_FT-STK500v2_FTWAKGHJ-if00-port0"
        .to_string();
    let mut stk = stk500v2::STK500v2::open(&port, SPECS).unwrap();
    println!("Programmer signature: {}", stk.read_programmer_signature()?);
    let mut isp: IspMode = stk.try_into()?;
    signature(&mut isp)?;
    fuses(&mut isp)?;
    lock_bytes(&mut isp)?;
    println!("reading flash");
    isp.read_flash(0, &mut flash)?;
    dump(&mut flash, String::from("flash.bin"));
    println!("reading eeprom");
    isp.read_eeprom(0, &mut eeprom)?;
    dump(&mut eeprom, String::from("eeprom.bin"));
    isp.close()?;
    return Ok(());
}

fn fuses<T: AVRFuseGet>(programmer: &mut T) -> Result<(), errors::ErrorKind> {
    println!("fuses: {}", programmer.get_fuses()?);
    Ok(())
}

fn lock_bytes<T: AVRLockByteGet>(programmer: &mut T) -> Result<(), errors::ErrorKind> {
    println!("lock byte {:#04X}", programmer.get_lock_byte()?);
    Ok(())
}

fn signature<T: MCUSignature>(programmer: &mut T) -> Result<(), errors::ErrorKind> {
    let sign = programmer.get_mcu_signature()?;
    println!(
        "MCU signature: {:#04x} {:#04x} {:#04x}",
        sign.bytes.0, sign.bytes.1, sign.bytes.2
    );
    Ok(())
}

fn dump(bytes: &mut Vec<u8>, name: String) {
    let found = bytes.iter().rposition(|&x| x != 0);
    let end = match found {
        Some(0) => 0,
        None => return,
        Some(x) => x + 1,
    };
    bytes.truncate(end);
    let mut file = File::create(name).unwrap();
    file.write_all(&bytes).unwrap();
}
