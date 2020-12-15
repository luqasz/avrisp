use crate::errors;
use crate::programmer;
use crate::specs;
use serial::core::{Error, SerialDevice, SerialPortSettings};
use std::convert::{TryFrom, TryInto};
use std::fmt;
use std::io::prelude::*;
use std::string::String;
use std::time::Duration;

#[allow(dead_code)]
mod command {

    pub enum Normal {
        SignOn = 0x01,
        SetParameter = 0x02,
        GetParameter = 0x03,
        SetDeviceParameters = 0x04,
        OSCcal = 0x05,
        LoadAddress = 0x06,
        FirmwareUpgrade = 0x07,
        SpiMulti = 0x1D,
        SetControlStack = 0x2D,
        EnterIspMode = 0x10,
        LeaveIspMode = 0x11,
    }

    impl Into<u8> for Normal {
        fn into(self) -> u8 {
            self as u8
        }
    }

    pub enum Isp {
        ChipErase = 0x12,
        ProgramFlash = 0x13,
        ReadFlash = 0x14,
        ProgramEeprom = 0x15,
        ReadEeprom = 0x16,
        ProgramFuse = 0x17,
        ReadFuse = 0x18,
        ProgramLock = 0x19,
        ReadLock = 0x1A,
        ReadSignature = 0x1B,
        ReadOsccal = 0x1C,
    }

    impl Into<u8> for Isp {
        fn into(self) -> u8 {
            self as u8
        }
    }
}

#[allow(dead_code)]
pub mod param {
    pub trait Readable {}

    pub trait Writable {}

    pub enum RO {
        BuildNumberLow = 0x80,
        BuildNumberHigh = 0x81,
        HwVer = 0x90,
        SwMajor = 0x91,
        SwMinor = 0x92,
        TopcardDetect = 0x9A, // This parameter only applies to STK500, not the AVRISP
        Status = 0x9C,
        Data = 0x9D,
    }

    impl Readable for RO {}

    impl From<RO> for u8 {
        fn from(value: RO) -> u8 {
            value as u8
        }
    }

    pub enum RW {
        Vtarget = 0x94,
        Vadjust = 0x95,
        OScPscale = 0x96,
        OscCmatch = 0x97,
        SckDuration = 0x98,
        ControllerInit = 0x9F,
        ResetPolarity = 0x9E,
    }

    impl Readable for RW {}

    impl Writable for RW {}

    impl From<RW> for u8 {
        fn from(value: RW) -> u8 {
            value as u8
        }
    }
}

#[allow(dead_code)]
pub enum Status {
    CmdOk = 0x00,
    CmdTimeout = 0x80,
    RdyBsyTout = 0x81,
    SetParamMissing = 0x82,
    CmdFailed = 0xC0,
    UnknownCmd = 0xC9,
    CheckSumError = 0xC1,
    AnswerChecksumError = 0xB0,
}

impl Into<u8> for Status {
    fn into(self) -> u8 {
        self as u8
    }
}

pub struct SwVersion {
    pub major: u8,
    pub minor: u8,
}

impl fmt::Display for SwVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor,)
    }
}

pub enum TopCard {
    STK501 = 0xAA,
    STK502 = 0x55,
    STK503 = 0xFA,
    STK504 = 0xEE,
    STK505 = 0xE4,
    STK520 = 0xDD,
}

/// Communication binary message with programmer.
///
/// Structure:
/// 1. Message start
/// 1. Sequence number. See [`SequenceGenerator`]
/// 1. Body length. Two bytes (maximum 275) in big endian order.
/// 1. Token
/// 1. Body
/// 1. Calculated checksum
#[derive(Debug)]
struct Message {
    buffer: MessageBuffer,
}

type MessageBuffer = [u8; Message::MAX_SIZE];

impl Message {
    const MESSAGE_START: u8 = 0x1B;
    const MESSAGE_START_POSITION: usize = 0;
    const TOKEN: u8 = 0x0E;
    const TOKEN_PSITION: usize = 4;
    const HEADER_SIZE: usize = 5;
    const CHECKSUM_SIZE: usize = 1;
    const BODY_START_POSITION: usize = 5;
    const LEN_BYTE_0_POSITION: usize = 2;
    const LEN_BYTE_1_POSITION: usize = 3;
    const SEQ_PSITION: usize = 1;
    const MAX_BODY_SIZE: usize = 275;
    const MAX_SIZE: usize = Self::MAX_BODY_SIZE + Self::CHECKSUM_SIZE + Self::HEADER_SIZE;

    fn new(seq: u8, body: Vec<u8>) -> Self {
        let mut buffer: MessageBuffer = [0; Self::MAX_SIZE];
        buffer[Self::MESSAGE_START_POSITION] = Self::MESSAGE_START;
        buffer[Self::TOKEN_PSITION] = Self::TOKEN;
        buffer[Self::SEQ_PSITION] = seq;
        let body_size = body.len();
        let end_index = Self::BODY_START_POSITION + body_size;
        buffer[Self::LEN_BYTE_0_POSITION..=Self::LEN_BYTE_1_POSITION]
            .swap_with_slice(&mut (body_size as u16).to_be_bytes());
        buffer[Self::BODY_START_POSITION..end_index].copy_from_slice(&body.as_slice());
        buffer[end_index] = Self::calc_checksum(&buffer[..end_index]);
        Self { buffer }
    }

    /// Return sequence number.
    fn get_sequence(&self) -> u8 {
        self.buffer[Self::SEQ_PSITION]
    }

    fn get_body_size(&self) -> u16 {
        u16::from_be_bytes([
            self.buffer[Self::LEN_BYTE_0_POSITION],
            self.buffer[Self::LEN_BYTE_1_POSITION],
        ])
    }

    /// Return ending index at where message ends. That is index including.
    fn get_end_index(&self) -> usize {
        Self::BODY_START_POSITION + self.get_body_size() as usize
    }

    fn body_slice(&self) -> &[u8] {
        &self.buffer[Self::BODY_START_POSITION..self.get_end_index()]
    }

    /// Return whole message as slice.
    fn as_slice(&self) -> &[u8] {
        &self.buffer[..=self.get_end_index()]
    }

    /// Calculate checksum (XOR of all bytes)
    fn calc_checksum(bytes: &[u8]) -> u8 {
        let mut result = bytes[0];
        for byte in bytes.iter().skip(1) {
            result ^= byte;
        }
        return result;
    }
}

impl TryFrom<MessageBuffer> for Message {
    type Error = errors::ErrorKind;

    fn try_from(buffer: MessageBuffer) -> Result<Self, Self::Error> {
        let body_size = u16::from_be_bytes([
            buffer[Self::LEN_BYTE_0_POSITION],
            buffer[Self::LEN_BYTE_1_POSITION],
        ]) as u16;
        let end_index = Self::BODY_START_POSITION + body_size as usize;
        let crc: u8 = buffer[end_index];
        if crc != Self::calc_checksum(&buffer[..end_index]) {
            return Err(errors::ErrorKind::ChecksumError);
        } else {
            Ok(Message { buffer })
        }
    }
}

fn to_hex(slice: &[u8]) -> String {
    let mut hexes: Vec<String> = Vec::with_capacity(slice.len());
    for i in slice {
        hexes.push(format!("{:#04x}", i));
    }
    return hexes.join(", ");
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "sequence_number={} body_length={} body=[{}]",
            self.get_sequence(),
            self.body_slice().len(),
            to_hex(self.body_slice()),
        )
    }
}

/// Incremented by one for each message sent.
/// Wraps to zero after 0xFF is reached.
struct SequenceGenerator {
    count: u8,
}

impl SequenceGenerator {
    fn new() -> SequenceGenerator {
        SequenceGenerator { count: 0 }
    }
}

impl Iterator for SequenceGenerator {
    type Item = u8;

    fn next(&mut self) -> Option<Self::Item> {
        let ret = Some(self.count);
        self.count = self.count.wrapping_add(1);
        ret
    }
}

pub struct STK500v2 {
    port: serial::SystemPort,
    sequencer: SequenceGenerator,
    specs: specs::Specs,
}

impl STK500v2 {
    pub fn open(port: &String, specs: specs::Specs) -> Result<STK500v2, Error> {
        let mut port = serial::open(&port)?;
        let mut settings = port.read_settings()?;

        settings.set_baud_rate(serial::Baud115200)?;
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_char_size(serial::Bits8);
        // Must be set to none.
        // Otherwise programmer may hang at random command.
        settings.set_flow_control(serial::FlowNone);

        port.write_settings(&settings)?;
        port.set_timeout(Duration::from_secs(1))?;
        Ok(STK500v2 {
            port,
            sequencer: SequenceGenerator::new(),
            specs,
        })
    }

    fn write_message(&mut self, msg: Message) -> Result<(), errors::ErrorKind> {
        self.port.write_all(msg.as_slice())?;
        self.port.flush()?;
        return Ok(());
    }

    fn read_message(&mut self) -> Result<Message, errors::ErrorKind> {
        let mut buffer: MessageBuffer = [0; Message::MAX_SIZE];
        self.port.read_exact(&mut buffer[..Message::HEADER_SIZE])?;
        let body_size = u16::from_be_bytes([
            buffer[Message::LEN_BYTE_0_POSITION],
            buffer[Message::LEN_BYTE_1_POSITION],
        ]) as usize;
        let end = Message::HEADER_SIZE + body_size + Message::CHECKSUM_SIZE;
        self.port
            .read_exact(&mut buffer[Message::BODY_START_POSITION..end])?;
        let msg = Message::try_from(buffer)?;
        return Ok(msg);
    }

    fn command(&mut self, body: Vec<u8>) -> Result<Message, errors::ErrorKind> {
        // This will always succeed
        let seq = self.sequencer.next().unwrap();
        let cmd = body[0];
        let sent_msg = Message::new(seq, body);
        self.write_message(sent_msg)?;
        let read_msg = self.read_message()?;

        if seq != read_msg.get_sequence() {
            return Err(errors::ErrorKind::SequenceError {});
        }
        if cmd != read_msg.body_slice()[0] {
            return Err(errors::ErrorKind::AnswerIdError {});
        }
        if read_msg.body_slice()[1] != Status::CmdOk.into() {
            return Err(errors::ErrorKind::StatusError {});
        }
        Ok(read_msg)
    }

    fn set_param<T>(&mut self, param: T, value: u8) -> Result<(), errors::ErrorKind>
    where
        T: param::Writable + Into<u8>,
    {
        let bytes = vec![command::Normal::SetParameter.into(), param.into(), value];
        let msg = self.command(bytes)?;
        if msg.body_slice()[0] != command::Normal::SetParameter.into() {
            return Err(errors::ErrorKind::AnswerIdError {});
        }
        if msg.body_slice()[1] != Status::CmdOk.into() {
            return Err(errors::ErrorKind::StatusError {});
        }
        Ok(())
    }

    fn get_param<T>(&mut self, param: T) -> Result<u8, errors::ErrorKind>
    where
        T: param::Readable + Into<u8>,
    {
        let bytes: Vec<u8> = vec![command::Normal::GetParameter.into(), param.into()];
        let msg = self.command(bytes)?;
        if msg.body_slice()[0] != command::Normal::GetParameter.into() {
            return Err(errors::ErrorKind::AnswerIdError {});
        }
        if msg.body_slice()[1] != Status::CmdOk.into() {
            return Err(errors::ErrorKind::StatusError {});
        }
        // return parameter
        Ok(msg.body_slice()[2])
    }

    pub fn read_programmer_signature(&mut self) -> Result<programmer::Variant, errors::ErrorKind> {
        let msg = self.command(vec![command::Normal::SignOn.into()])?;
        let variant = String::from_utf8(msg.body_slice()[3..].to_vec())?;
        Ok(programmer::Variant::try_from(variant)?)
    }
}

impl TryInto<IspMode> for STK500v2 {
    type Error = errors::ErrorKind;
    fn try_into(mut self) -> Result<IspMode, Self::Error> {
        let bytes = vec![
            command::Normal::EnterIspMode.into(),
            self.specs.timeout,
            self.specs.stab_delay,
            self.specs.cmd_exe_delay,
            self.specs.synch_loops,
            self.specs.byte_delay,
            self.specs.pool_value,
            self.specs.pool_index,
            specs::PROGRAMMING_ENABLE.0,
            specs::PROGRAMMING_ENABLE.1,
            specs::PROGRAMMING_ENABLE.2,
            specs::PROGRAMMING_ENABLE.3,
        ];
        self.set_param(param::RW::ResetPolarity, self.specs.reset_polarity.into())?;
        self.command(bytes)?;
        Ok(IspMode::new(self))
    }
}

pub struct IspMode {
    prog: STK500v2,
}

impl IspMode {
    fn new(prog: STK500v2) -> IspMode {
        IspMode { prog }
    }

    fn load_address(&mut self, address: usize) -> Result<(), errors::ErrorKind> {
        let mut dst_addr = vec![command::Normal::LoadAddress.into()];
        dst_addr.extend((address as u32).to_be_bytes().to_vec());
        self.prog.command(dst_addr)?;
        Ok(())
    }

    fn read_flash_command(
        &mut self,
        size: usize,
        buffer: &mut [u8],
    ) -> Result<(), errors::ErrorKind> {
        let size_bytes = (size as u16).to_be_bytes();
        let msg = self.prog.command(vec![
            command::Isp::ReadFlash.into(),
            size_bytes[0],
            size_bytes[1],
            // Stk500v2 firmware handles selecting low/high byte when reading.
            specs::READ_FLASH_LOW.0,
        ])?;
        let data_offset = 2;
        buffer.copy_from_slice(&msg.body_slice()[data_offset..(size + data_offset)]);
        Ok(())
    }

    fn read_eeprom_command(
        &mut self,
        size: usize,
        buffer: &mut [u8],
    ) -> Result<(), errors::ErrorKind> {
        let size_bytes = (size as u16).to_be_bytes();
        let msg = self.prog.command(vec![
            command::Isp::ReadEeprom.into(),
            size_bytes[0],
            size_bytes[1],
            specs::READ_EEPROM.0,
        ])?;
        let data_offset = 2;
        buffer.copy_from_slice(&msg.body_slice()[data_offset..(size + data_offset)]);
        Ok(())
    }

    fn read_fuse(&mut self, cmd: specs::IspCommand) -> Result<u8, errors::ErrorKind> {
        let msg = self.prog.command(vec![
            command::Isp::ReadFuse.into(),
            self.prog.specs.fuse_poll_index,
            cmd.0,
            cmd.1,
            cmd.2,
            cmd.3,
        ])?;
        Ok(msg.body_slice()[2])
    }
}

impl programmer::FlashRead for IspMode {
    // Does not work on atmega2560.
    // Requires some kind of different handling when loading memory address
    fn read(&mut self, buffer: &mut [u8]) -> Result<(), errors::ErrorKind> {
        let size = self.prog.specs.flash.page_size;
        // Stk500v2 firmware handles incrementing address on its own.
        // Reduces reading time since no load address command needs to be send.
        self.load_address(0)?;
        for addr in (0..buffer.len()).step_by(size) {
            self.read_flash_command(size, &mut buffer[addr..(addr + size)])?;
        }
        Ok(())
    }
}

impl programmer::EEPROMRead for IspMode {
    fn read(&mut self, bytes: &mut [u8]) -> Result<(), errors::ErrorKind> {
        // According to AVR068 PDF, LoadAddress command needs to be executed once.
        // Firmware will increment address on its own. At least in byte mode.
        //
        // Tested on stk500v2 programmer, which reduced whole reading time by half.
        let size = self.prog.specs.eeprom.page_size;
        self.load_address(0)?;
        for addr in (0..bytes.len()).step_by(size) {
            self.read_eeprom_command(size, &mut bytes[addr..(addr + size)])?;
        }
        Ok(())
    }
}

impl programmer::Erase for IspMode {
    fn erase(&mut self) -> Result<(), errors::ErrorKind> {
        self.prog.command(vec![
            command::Isp::ChipErase.into(),
            self.prog.specs.erase_delay,
            self.prog.specs.erase_poll_method,
            specs::CHIP_ERASE.0,
            specs::CHIP_ERASE.1,
            specs::CHIP_ERASE.2,
            specs::CHIP_ERASE.3,
        ])?;
        Ok(())
    }
}

impl programmer::Programmer for IspMode {
    fn close(mut self) -> Result<(), errors::ErrorKind> {
        let bytes = vec![
            command::Normal::LeaveIspMode.into(),
            self.prog.specs.pre_delay,
            self.prog.specs.post_delay,
        ];
        self.prog.command(bytes)?;
        Ok(())
    }
}

impl programmer::AVRLockByteGet for IspMode {
    fn get_lock_byte(&mut self) -> Result<u8, errors::ErrorKind> {
        let msg = self.prog.command(vec![
            command::Isp::ReadLock.into(),
            self.prog.specs.lock_poll_index,
            specs::READ_LOCK.0,
            specs::READ_LOCK.1,
            specs::READ_LOCK.2,
            specs::READ_LOCK.3,
        ])?;
        Ok(msg.body_slice()[2])
    }
}

impl programmer::AVRFuseGet for IspMode {
    fn get_fuses(&mut self) -> Result<programmer::AVRFuse, errors::ErrorKind> {
        Ok(programmer::AVRFuse {
            low: self.read_fuse(specs::READ_LOW_FUSE)?,
            high: self.read_fuse(specs::READ_HIGH_FUSE)?,
            extended: self.read_fuse(specs::READ_EXTENDED_FUSE)?,
        })
    }
}

impl programmer::MCUSignature for IspMode {
    fn get_mcu_signature(&mut self) -> Result<specs::Signature, errors::ErrorKind> {
        let mut signature: [u8; 3] = [0; 3];
        for addr in 0..signature.len() {
            let msg = self.prog.command(vec![
                command::Isp::ReadSignature.into(),
                self.prog.specs.signature_poll_index,
                specs::READ_SIGNATURE.0,
                specs::READ_SIGNATURE.1,
                addr as u8,
                specs::READ_SIGNATURE.3,
            ])?;
            signature[addr] = msg.body_slice()[2];
        }
        Ok(specs::Signature::from(signature))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claim::*;

    mod sequence_generator {

        use super::*;

        #[test]
        fn overflows_after_255() {
            let mut gen = SequenceGenerator::new().skip(256);
            assert_eq!(gen.next(), Some(0));
        }

        #[test]
        fn starts_at_0() {
            let mut gen = SequenceGenerator::new();
            assert_eq!(gen.next(), Some(0));
        }

        #[test]
        fn increments_by_1() {
            let mut gen = SequenceGenerator::new().skip(1);
            assert_eq!(gen.next(), Some(1));
        }
    }

    mod message {
        use super::*;

        #[test]
        fn calculates_checksum() {
            assert_eq!(Message::calc_checksum(&[2, 55, 22, 78]), 109);
        }

        #[test]
        fn try_from_array_is_ok() {
            let mut buffer: MessageBuffer = [0; Message::MAX_SIZE];
            buffer[0] = Message::MESSAGE_START;
            buffer[1] = 1;
            buffer[2] = 0;
            buffer[3] = 4;
            buffer[4] = Message::TOKEN;
            buffer[5] = 89;
            buffer[6] = 100;
            buffer[7] = 78;
            buffer[8] = 109;
            buffer[9] = 14;
            assert_ok!(Message::try_from(buffer));
        }
        #[test]
        fn try_from_array_bad_checksum() {
            let mut buffer: MessageBuffer = [0; Message::MAX_SIZE];
            buffer[0] = Message::MESSAGE_START;
            buffer[1] = 1;
            buffer[2] = 0;
            buffer[3] = 4;
            buffer[4] = Message::TOKEN;
            buffer[5] = 89;
            buffer[6] = 100;
            buffer[7] = 78;
            buffer[8] = 109;
            buffer[9] = 0;
            let err = Message::try_from(buffer).unwrap_err();
            match err {
                errors::ErrorKind::ChecksumError => (),
                _ => panic!("wrong error returned"),
            };
        }
    }
}
