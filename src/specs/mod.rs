pub mod atmega;

pub type IspCommand = (u8, u8, u8, u8);

pub const PROGRAMMING_ENABLE: IspCommand = (0xac, 0x53, 0x00, 0x00);
pub const CHIP_ERASE: IspCommand = (0xac, 0x80, 0x00, 0x00);
pub const READ_SIGNATURE: IspCommand = (0x30, 0x00, 0x00, 0x00);
pub const READ_FLASH_HIGH: IspCommand = (0x28, 0x00, 0x00, 0x00);
pub const READ_FLASH_LOW: IspCommand = (0x20, 0x00, 0x00, 0x00);
pub const READ_EEPROM: IspCommand = (0xa0, 0x00, 0x00, 0x00);
pub const READ_LOW_FUSE: IspCommand = (0x50, 0x00, 0x00, 0x00);
pub const READ_HIGH_FUSE: IspCommand = (0x58, 0x08, 0x00, 0x00);
pub const READ_EXTENDED_FUSE: IspCommand = (0x50, 0x08, 0x00, 0x00);
pub const READ_LOCK: IspCommand = (0x58, 0x00, 0x00, 0x00);
pub const READ_OSCCAL: IspCommand = (0x38, 0x00, 0x00, 0x00);
pub const LOAD_EXTENDED_ADDRESS: IspCommand = (0x4d, 0x00, 0x00, 0x00);
pub const LOAD_FLASH_PAGE_HIGH: IspCommand = (0x48, 0x00, 0x00, 0x00);
pub const LOAD_FLASH_PAGE_LOW: IspCommand = (0x40, 0x00, 0x00, 0x00);
pub const LOAD_EEPROM_PAGE: IspCommand = (0xc1, 0x00, 0x00, 0x00);
pub const WRITE_FLASH: IspCommand = (0x4c, 0x00, 0x00, 0x00);
pub const WRITE_EEPROM: IspCommand = (0xc0, 0x00, 0x00, 0x00);
pub const WRITE_EEPROM_PAGE: IspCommand = (0xc2, 0x00, 0x00, 0x00);
pub const WRITE_LOW_FUSE: IspCommand = (0xac, 0xa0, 0x00, 0x00);
pub const WRITE_HIGH_FUSE: IspCommand = (0xac, 0xa8, 0x00, 0x00);
pub const WRITE_EXTENDED_FUSE: IspCommand = (0xac, 0xa4, 0x00, 0x00);
pub const WRITE_LOCK: IspCommand = (0xac, 0xe0, 0x00, 0x00);

/// AVR MCU signature.
pub struct Signature {
    pub bytes: (u8, u8, u8),
}

impl From<[u8; 3]> for Signature {
    fn from(bytes: [u8; 3]) -> Signature {
        Signature {
            bytes: (bytes[0], bytes[1], bytes[2]),
        }
    }
}

impl From<(u8, u8, u8)> for Signature {
    fn from(tup: (u8, u8, u8)) -> Signature {
        Signature { bytes: tup }
    }
}

/// Memory segment e.g. EEPROM, flash.
#[allow(dead_code)]
pub struct Memory {
    pub start: usize, // Start address of a given memory section. Given in XML in address-spaces section
    pub size: usize,  // Total number of bytes. Given in XML in address-spaces section
    pub page_size: usize, // Page size in bytes. Given in XML in address-spaces section
    pub mode: usize,
    pub delay: usize,
}

/// MCU specs used for programming.
///
/// Can be obtained from atdf files found in atpack on http://packs.download.atmel.com/
pub struct Specs {
    pub timeout: u8,
    pub stab_delay: u8,
    pub cmd_exe_delay: u8,
    pub synch_loops: u8,
    pub byte_delay: u8,
    pub pool_value: u8,
    pub pool_index: u8,
    pub pre_delay: u8,
    pub post_delay: u8,
    pub reset_polarity: bool,
    pub erase_poll_method: u8,
    pub erase_delay: u8,
    pub signature: Signature,
    pub fuse_poll_index: u8,
    pub lock_poll_index: u8,
    pub osccal_poll_index: u8,
    pub signature_poll_index: u8,
    pub flash: Memory,
    pub eeprom: Memory,
}

#[cfg(test)]
mod tests {
    use super::Signature;

    #[test]
    fn signature_from_array_trait() {
        let sign = Signature::from([1, 2, 3]);
        assert_eq!(sign.bytes.0, 1);
        assert_eq!(sign.bytes.1, 2);
        assert_eq!(sign.bytes.2, 3);
    }

    #[test]
    fn signature_from_tuple_trait() {
        let sign = Signature::from((1, 2, 3));
        assert_eq!(sign.bytes.0, 1);
        assert_eq!(sign.bytes.1, 2);
        assert_eq!(sign.bytes.2, 3);
    }
}
