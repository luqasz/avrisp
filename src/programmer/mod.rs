#[allow(dead_code)]
pub mod stk500v2;
use crate::errors;
use std::convert::TryFrom;
use std::fmt;

pub struct AVRFuse {
    low: u8,
    high: u8,
    extended: u8,
}

impl fmt::Display for AVRFuse {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        return write!(
            f,
            "low: {:#04X} high: {:#04X} extended: {:#04X}",
            self.low, self.high, self.extended,
        );
    }
}

#[allow(non_camel_case_types)]
pub enum Variant {
    STK500_V2,
    AVRISP_2,
}

impl fmt::Display for Variant {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Variant::STK500_V2 => write!(f, "STK 500 v2"),
            Variant::AVRISP_2 => write!(f, "AVR ISP 2"),
        }
    }
}

impl TryFrom<String> for Variant {
    type Error = errors::UnknownProgrammer;
    fn try_from(string: String) -> Result<Self, Self::Error> {
        match string.as_ref() {
            "STK500_2" => Ok(Variant::STK500_V2),
            "AVRISP_2" => Ok(Variant::AVRISP_2),
            _ => Err(errors::UnknownProgrammer {}),
        }
    }
}

pub trait Programmer {
    // Close and release all resources.
    fn close(self) -> Result<(), errors::ErrorKind>;
}

// Perform full chip erase including EEPROM and flash.
pub trait Erase {
    fn erase(&mut self) -> Result<(), errors::ErrorKind>;
}

pub trait AVRFuseGet {
    fn get_fuses(&mut self) -> Result<AVRFuse, errors::ErrorKind>;
}

pub trait AVRFuseSet {
    fn set_fuses(&mut self, fuses: &AVRFuse) -> Result<AVRFuse, errors::ErrorKind>;
}

pub trait AVRLockByteGet {
    fn get_lock_byte(&mut self) -> Result<u8, errors::ErrorKind>;
}

pub trait AVRLockByteSet {
    fn set_lock_byte(&mut self, byte: u8) -> Result<u8, errors::ErrorKind>;
}

pub trait MCUSignature {
    fn get_mcu_signature(&mut self) -> Result<avrisp::Signature, errors::ErrorKind>;
}

pub trait EEPROMRead {
    fn read(&mut self, bytes: &mut [u8]) -> Result<(), errors::ErrorKind>;
}
