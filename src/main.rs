pub mod errors;
pub mod programmer;
use avrisp;
use programmer::stk500v2;
use programmer::stk500v2::IspMode;
use programmer::{AVRFuse, AVRProgrammer};
use std::convert::{TryFrom, TryInto};
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let mut eeprom = [0; 1024];
    let mut flash = [0; 4096];
    let port = "/dev/serial/by-id/usb-microSENSE_USB_AVR_ISP_II_FT-STK500v2_FTWAKGHJ-if00-port0"
        .to_string();
    let mut stk = stk500v2::STK500v2::open(&port, avrisp::atmega::ATMEGA_32).unwrap();
    println!(
        "Programmer signature: {}",
        stk.read_programmer_signature().unwrap()
    );
    let mut isp: IspMode = stk.try_into().unwrap();
    let sign = isp.get_mcu_signature().unwrap();
    println!(
        "MCU signature: {:#04x} {:#04x} {:#04x}",
        sign.bytes.0, sign.bytes.1, sign.bytes.2
    );
    isp.read_eeprom(0, &mut eeprom).unwrap();
    dump(&eeprom, String::from("eeprom.bin"));
    isp.read_flash(0, &mut flash).unwrap();
    dump(&flash, String::from("flash.bin"));
    println!("low fuse: {}", isp.get_fuses().unwrap());
    println!("lock byte {:#04X}", isp.get_lock_byte().unwrap());
}

fn dump(bytes: &[u8], name: String) {
    let mut file = File::create(name).unwrap();
    file.write_all(&bytes).unwrap();
}
