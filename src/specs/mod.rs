/// All listed specifications and parameters come from atdf files.
/// Can be obtained [here](http://packs.download.atmel.com/). Those are
/// ZIPs with xml files describing given MCU. Simmilar to SVD for ARM.
pub mod atmega;

/// Low level ISP command. Can be found in chip datasheet.
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

/// MCU signature.
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

/// Memory segment. EEPROM or flash.
pub struct Memory {
    /// Start address of a given memory section. Given in XML in `address-spaces` section
    pub start: usize,
    /// Total number of bytes. Given in XML in `address-spaces` section
    pub size: usize,
    /// Page size in bytes. Given in XML in `address-spaces` section
    pub page_size: usize,
    /// In xml:
    /// * `ISP_INTERFACE/IspProgramFlash_mode` for flash.
    /// * `ISP_INTERFACE/IspProgramEeprom_mode` for eeprom.
    pub mode: usize,
    /// In xml:
    /// * `ISP_INTERFACE/IspProgramFlash_delay` for flash.
    /// * `ISP_INTERFACE/IspProgramEeprom_delay` for eeprom.
    pub delay: usize,
}

/// Parameters required by programmers.
///
/// Can be found in xml (under `ISP_INTERFACE`) for given chip.
pub struct Specs {
    /// `IspEnterProgMode_timeout`
    pub timeout: u8,
    /// `IspEnterProgMode_stabDelay`
    pub stab_delay: u8,
    /// `IspEnterProgMode_cmdexeDelay`
    pub cmd_exe_delay: u8,
    /// `IspEnterProgMode_synchLoops`
    pub synch_loops: u8,
    /// `IspEnterProgMode_byteDelay`
    pub byte_delay: u8,
    /// `IspEnterProgMode_pollValue`
    pub pool_value: u8,
    /// `IspEnterProgMode_pollIndex`
    pub pool_index: u8,
    /// `IspLeaveProgMode_preDelay`
    pub pre_delay: u8,
    /// `IspLeaveProgMode_postDelay`
    pub post_delay: u8,
    pub reset_polarity: bool,
    /// `IspChipErase_pollMethod`
    pub erase_poll_method: u8,
    /// `IspChipErase_eraseDelay`
    pub erase_delay: u8,
    pub signature: Signature,
    /// `IspReadFuse_pollIndex`
    pub fuse_poll_index: u8,
    /// `IspReadLock_pollIndex`
    pub lock_poll_index: u8,
    /// `IspReadOsccal_pollIndex`
    pub osccal_poll_index: u8,
    /// `IspReadSign_pollIndex`
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
