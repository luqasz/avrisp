use crate::errors;
use crate::programmer;
use avrisp;
use serial::core::{Error, SerialDevice, SerialPortSettings};
use std::fmt;
use std::io;
use std::io::prelude::*;
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
    pub trait Readable {
        fn cast(self) -> u8;
    }

    pub trait Writable {
        fn cast(self) -> u8;
    }

    pub enum RO {
        BuildNumberLow = 0x80,
        BuildNumberHigh = 0x81,
        HwVer = 0x90,
        SwMajor = 0x91,
        SwMinor = 0x92,
        TopcardDetect = 0x9A,
        Status = 0x9C,
        Data = 0x9D,
    }

    impl Readable for RO {
        fn cast(self) -> u8 {
            self as u8
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

    impl Readable for RW {
        fn cast(self) -> u8 {
            self as u8
        }
    }

    impl Writable for RW {
        fn cast(self) -> u8 {
            self as u8
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

/// Message structure:
/// MESSAGE_START (always 0x1b)
/// Sequence number (u8 incremented for each message sent, overflows after 0xff)
/// Body length (STK500 firmware can only handle body with maximum of 275 bytes, in big endian order)
/// TOKEN (always 0x0e)
/// Body as bytes
/// Calculated checksum (XOR of all bytes in message)
struct Message {
    seq: u8,
    len: u16,
    body: Vec<u8>,
}

impl Message {
    /// Constant fields within message.
    /// Same for reading and writing.
    const MESSAGE_START: u8 = 0x1B;
    const TOKEN: u8 = 0x0E;

    fn from_body(seq: u8, body: Vec<u8>) -> Self {
        Message {
            len: body.len() as u16,
            body: body.to_vec(),
            seq: seq,
        }
    }

    /// Construct Message fron slice of bytes.
    /// Bytes can not contain checksum.
    fn from_bytes(bytes: &[u8]) -> Self {
        Message {
            len: u16::from_be_bytes([bytes[2], bytes[3]]) as u16,
            body: bytes[5..].to_vec(),
            seq: bytes[1],
        }
    }

    fn pack(&self) -> Vec<u8> {
        let len = (self.len as u16).to_be_bytes();
        let mut bytes: Vec<u8> = vec![
            Message::MESSAGE_START,
            self.seq,
            len[0],
            len[1],
            Message::TOKEN,
        ];
        bytes.extend_from_slice(&self.body);
        bytes.push(self.checksum());
        bytes
    }

    /// Calculate checksum (XOR of all fields in message)
    fn checksum(&self) -> u8 {
        let mut result = Message::MESSAGE_START;
        result ^= self.seq;
        for byte in self.len.to_be_bytes().iter() {
            result ^= byte;
        }
        result ^= Message::TOKEN;
        for byte in self.body.iter() {
            result ^= byte;
        }
        result
    }
}

impl fmt::Display for Message {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "seq: {} len: {} checksum: {} body: {:?}",
            self.seq,
            self.len,
            self.checksum(),
            self.body
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

pub struct Normal {
    port: serial::SystemPort,
    sequencer: SequenceGenerator,
}

impl Normal {
    pub fn open(port: &String) -> Result<Normal, Error> {
        let mut port = serial::open(&port)?;
        let mut settings = port.read_settings()?;
        settings.set_baud_rate(serial::Baud115200)?;
        settings.set_parity(serial::ParityNone);
        settings.set_stop_bits(serial::Stop1);
        settings.set_char_size(serial::Bits8);
        port.write_settings(&settings)?;
        port.set_timeout(Duration::from_secs(1))?;
        Ok(Normal {
            port: port,
            sequencer: SequenceGenerator::new(),
        })
    }

    fn write_message(&mut self, msg: Message) -> Result<(), io::Error> {
        self.port.write_all(&msg.pack().as_slice())
    }

    fn read_message(&mut self) -> Result<Message, errors::ErrorKind> {
        let mut buf = vec![0u8; 5];
        self.port.read_exact(&mut buf)?;
        let len = u16::from_be_bytes([buf[2], buf[3]]) as usize;
        // Extend to fit body and checksum byte.
        buf.resize(buf.len() + len + 1, 0);
        self.port.read_exact(&mut buf[5..])?;
        // Remove last byte which is a checksum.
        let read_checksum = buf.pop().unwrap();

        let msg = Message::from_bytes(&buf);
        if msg.checksum() != read_checksum {
            return Err(errors::ErrorKind::ChecksumError {});
        }
        Ok(msg)
    }

    fn cmd(&mut self, cmd: u8, mut body: Vec<u8>) -> Result<Message, errors::ErrorKind> {
        let seq = self.sequencer.next().unwrap();
        body.insert(0, cmd);
        let sent_msg = Message::from_body(seq, body);
        println!("writing: {}", sent_msg);
        self.write_message(sent_msg)?;

        let read_msg = self.read_message()?;

        if seq != read_msg.seq {
            return Err(errors::ErrorKind::SequenceError {});
        }
        if cmd != read_msg.body[0] {
            return Err(errors::ErrorKind::AnswerIdError {});
        }
        if read_msg.body[1] != Status::CmdOk.into() {
            return Err(errors::ErrorKind::StatusError {});
        }
        println!("reading: {}", read_msg);
        Ok(read_msg)
    }

    pub fn set_param<T: param::Writable>(
        &mut self,
        param: T,
        value: u8,
    ) -> Result<(), errors::ErrorKind> {
        let bytes = vec![param.cast(), value];
        self.cmd(command::Normal::SetParameter.into(), bytes)?;
        Ok(())
    }

    pub fn get_param<T: param::Readable>(&mut self, param: T) -> Result<u8, errors::ErrorKind> {
        let bytes = vec![param.cast()];
        let msg = self.cmd(command::Normal::GetParameter.into(), bytes)?;
        Ok(msg.body[1])
    }

    pub fn read_programmer_signature(&mut self) -> Result<programmer::Variant, errors::ErrorKind> {
        let msg = self.cmd(command::Normal::SignOn.into(), vec![])?;
        let variant = String::from_utf8(msg.body[3..].to_vec()).unwrap();
        match variant.as_ref() {
            "STK500_2" => Ok(programmer::Variant::STK500_V2),
            "AVRISP_2" => Ok(programmer::Variant::AVRISP_2),
            _ => panic!("Unknown programmer type"),
        }
    }

    fn set_reset_polarity(&mut self, polarity: bool) -> Result<(), errors::ErrorKind> {
        self.set_param(param::RW::ResetPolarity, polarity as u8)?;
        Ok(())
    }

    pub fn isp_mode(mut self, specs: avrisp::Specs) -> Result<IspMode, errors::ErrorKind> {
        let bytes = vec![
            specs.timeout,
            specs.stab_delay,
            specs.cmd_exe_delay,
            specs.synch_loops,
            specs.byte_delay,
            specs.pool_value,
            specs.pool_index,
            avrisp::PROGRAMMING_ENABLE.0,
            avrisp::PROGRAMMING_ENABLE.1,
            avrisp::PROGRAMMING_ENABLE.2,
            avrisp::PROGRAMMING_ENABLE.3,
        ];
        self.set_reset_polarity(specs.reset_polarity)?;
        self.cmd(command::Normal::EnterIspMode.into(), bytes)?;
        Ok(IspMode::new(self, specs))
    }
}

pub struct IspMode {
    prog: Normal,
    specs: avrisp::Specs,
}

impl IspMode {
    fn new(prog: Normal, specs: avrisp::Specs) -> IspMode {
        IspMode {
            prog: prog,
            specs: specs,
        }
    }

    pub fn erase(&mut self) -> Result<(), errors::ErrorKind> {
        self.prog.cmd(
            command::Isp::ChipErase.into(),
            vec![
                self.specs.erase_delay,
                self.specs.erase_poll_method,
                avrisp::CHIP_ERASE.0,
                avrisp::CHIP_ERASE.1,
                avrisp::CHIP_ERASE.2,
                avrisp::CHIP_ERASE.3,
            ],
        )?;
        Ok(())
    }

    pub fn read_flash(&mut self, start: usize, bytes: &mut [u8]) -> Result<(), errors::ErrorKind> {
        // For word-addressed memories (program flash), the Address parameter is the word address.
        let addr_step = self.specs.flash.page_size / 2;
        for (addr, buffer) in (start..bytes.len())
            .step_by(addr_step)
            .zip(bytes.chunks_exact_mut(self.specs.flash.page_size))
        {
            self.prog.cmd(
                command::Normal::LoadAddress.into(),
                (addr as u32).to_be_bytes().to_vec(),
            )?;
            let length_bytes = (self.specs.flash.page_size as u16).to_be_bytes();
            let mut msg = self.prog.cmd(
                command::Isp::ReadFlash.into(),
                vec![length_bytes[0], length_bytes[1], avrisp::READ_FLASH_LOW.0],
            )?;
            let data_offset = 2;
            buffer.swap_with_slice(
                &mut msg.body[data_offset..(self.specs.flash.page_size + data_offset)],
            );
        }
        Ok(())
    }

    pub fn read_eeprom(&mut self, start: usize, bytes: &mut [u8]) -> Result<(), errors::ErrorKind> {
        for (addr, buffer) in (start..(bytes.len() + start))
            .step_by(self.specs.eeprom.page_size)
            .zip(bytes.chunks_exact_mut(self.specs.eeprom.page_size))
        {
            self.prog.cmd(
                command::Normal::LoadAddress.into(),
                (addr as u32).to_be_bytes().to_vec(),
            )?;
            let length_bytes = (self.specs.eeprom.page_size as u16).to_be_bytes();
            let mut msg = self.prog.cmd(
                command::Isp::ReadEeprom.into(),
                vec![length_bytes[0], length_bytes[1], avrisp::READ_EEPROM.0],
            )?;
            let data_offset = 2;
            buffer.swap_with_slice(
                &mut msg.body[data_offset..(self.specs.eeprom.page_size + data_offset)],
            );
        }
        Ok(())
    }

    fn read_fuse(&mut self, cmd: avrisp::IspCommand) -> Result<u8, errors::ErrorKind> {
        let msg = self.prog.cmd(
            command::Isp::ReadFuse.into(),
            vec![self.specs.fuse_poll_index, cmd.0, cmd.1, cmd.2, cmd.3],
        )?;
        Ok(msg.body[2])
    }

    pub fn read_low_fuse(&mut self) -> Result<u8, errors::ErrorKind> {
        self.read_fuse(avrisp::READ_LOW_FUSE)
    }

    pub fn read_high_fuse(&mut self) -> Result<u8, errors::ErrorKind> {
        self.read_fuse(avrisp::READ_HIGH_FUSE)
    }

    pub fn read_extended_fuse(&mut self) -> Result<u8, errors::ErrorKind> {
        self.read_fuse(avrisp::READ_EXTENDED_FUSE)
    }

    pub fn read_lock_byte(&mut self) -> Result<u8, errors::ErrorKind> {
        let msg = self.prog.cmd(
            command::Isp::ReadLock.into(),
            vec![
                self.specs.lock_poll_index,
                avrisp::READ_LOCK.0,
                avrisp::READ_LOCK.1,
                avrisp::READ_LOCK.2,
                avrisp::READ_LOCK.3,
            ],
        )?;
        Ok(msg.body[2])
    }

    pub fn read_mcu_signature(&mut self) -> Result<avrisp::Signature, errors::ErrorKind> {
        let mut signature: [u8; 3] = [0; 3];
        for addr in 0..signature.len() {
            let msg = self.prog.cmd(
                command::Isp::ReadSignature.into(),
                vec![
                    self.specs.signature_poll_index,
                    avrisp::READ_SIGNATURE.0,
                    avrisp::READ_SIGNATURE.1,
                    addr as u8,
                    avrisp::READ_SIGNATURE.3,
                ],
            )?;
            signature[addr] = msg.body[2];
        }
        Ok(avrisp::Signature::from(signature))
    }
}

impl Drop for IspMode {
    fn drop(&mut self) {
        let bytes = vec![self.specs.pre_delay, self.specs.post_delay];
        self.prog
            .cmd(command::Normal::LeaveIspMode.into(), bytes)
            .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::SequenceGenerator;

    mod sequence_generator {

        use super::SequenceGenerator;

        #[test]
        fn overflows_after_255() {
            let mut gen = SequenceGenerator { count: 255 }.skip(1);
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
}
