pub mod errors;
pub mod programmer;
use avrisp;
use programmer::stk500v2;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    let mut eeprom = [0; 1024];
    let mut flash = [0; 4096];
    let port = "/dev/tty.usbserial-FTWAKGHJ".to_string();
    let mut stk = stk500v2::Normal::open(&port).unwrap();
    println!(
        "Programmer signature: {}",
        stk.read_programmer_signature().unwrap()
    );
    let mut isp = stk.isp_mode(avrisp::atmega::ATMEGA_32).unwrap();
    let sign = isp.read_mcu_signature().unwrap();
    println!(
        "MCU signature: {:#04x} {:#04x} {:#04x}",
        sign.bytes.0, sign.bytes.1, sign.bytes.2
    );
    isp.read_eeprom(0, &mut eeprom).unwrap();
    dump(&eeprom, String::from("eeprom.bin"));
    isp.read_flash(0, &mut flash).unwrap();
    dump(&flash, String::from("flash.bin"));
    println!("low fuse: {:#04X}", isp.read_low_fuse().unwrap());
    println!("high fuse: {:#04X}", isp.read_high_fuse().unwrap());
    println!("extended fuse: {:#04X}", isp.read_extended_fuse().unwrap());
    println!("lock byte {:#04X}", isp.read_lock_byte().unwrap());
}

fn dump(bytes: &[u8], name: String) {
    let mut file = File::create(name).unwrap();
    file.write_all(&bytes).unwrap();
}
