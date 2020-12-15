/// All listed specifications and parameters come from atdf files.
/// Can be obtained [here](http://packs.download.atmel.com/). Those are
/// ZIPs with xml files describing given MCU. Simmilar to SVD for ARM.
pub mod atmega;
use std::fmt;

/// MCU signature.
#[derive(PartialEq, Debug)]
pub struct Signature {
    pub bytes: (u8, u8, u8),
}

impl fmt::Display for Signature {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{:#04x} {:#04x} {:#04x}",
            self.bytes.0, self.bytes.1, self.bytes.2
        )
    }
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
    use super::*;

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

    #[test]
    fn signature_equality() {
        let s1 = Signature { bytes: (1, 2, 3) };
        let s2 = Signature { bytes: (1, 2, 3) };
        assert_eq!(s1, s2);
    }

    #[test]
    fn signature_not_equal() {
        let s1 = Signature { bytes: (1, 2, 3) };
        let s2 = Signature { bytes: (1, 2, 2) };
        assert_ne!(s1, s2);
    }
}
