#![forbid(unsafe_code)]

use byteorder::ReadBytesExt;
use std::io::{self, BufRead};
use std::ops::{Add, AddAssign};
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BitSequence {
    bits: u16,
    len: u8,
}

impl BitSequence {
    pub fn new(bits: u16, len: u8) -> Self {
        // NB: make sure to zero unused bits so that Eq and Hash work as expected.
        // TODO: your code goes here.
        Self {
            bits,
            len,
        }
    }

    pub fn bits(&self) -> u16 {
        // TODO: your code goes here.
        self.bits
    }

    pub fn len(&self) -> u8 {
        // TODO: your code goes here.
        self.len
    }

    pub fn concat(self, other: Self) -> Self {
        let len = self.len + other.len;
        let sbits = other.bits | (self.bits << other.len);
        Self { len, bits: sbits }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct BitReader<T> {
    stream: T,
    current_byte: u8,
    current_index: i8,
    // TODO: your code goes here.
}

impl<T: BufRead> BitReader<T> {
    pub fn new(stream: T) -> Self {
        Self {
            stream,
            current_byte: 0,
            current_index: -1,
        }
    }

    pub fn read_u32(&mut self) -> io::Result<u32> {
        let mut out = 0u32;
        let mut cur_pos = 0;
        let len = 32;
        loop {
            if self.current_index == -1 || self.current_index == 8 {
                self.current_byte = self.stream.read_u8()?;
                self.current_index = 0;
            }
            (out, cur_pos) = self.read_from_buf_u32(len, out, cur_pos);
            if cur_pos == len {
                return Ok(out);
            }
        }
    }
    pub fn read_str_to_null(&mut self) -> io::Result<String> {
        let mut buf = Vec::new();
        self.stream.read_until(u8::try_from('\0').unwrap(), &mut buf)?;
        match String::from_utf8(buf) {
            Ok(o) => { Ok(o) }
            Err(_) => { Err(io::Error::from(io::ErrorKind::UnexpectedEof)) }
        }
    }

    pub fn read_bits(&mut self, mut len: u8) -> io::Result<BitSequence> {
        if len == 0 {
            return Ok(BitSequence::new(0, 0));
        }
        let mut out = 0;
        let mut cur_pos = 0;
        loop {
            if self.current_index == -1 || self.current_index == 8 {
                self.current_byte = self.stream.read_u8()?;
                self.current_index = 0;
            }
            (out, cur_pos) = self.read_from_buf_u16(len, out, cur_pos);
            if cur_pos == len {
                return Ok(BitSequence::new(out, len));
            }
        }
    }

    fn read_from_buf_u32(&mut self, len: u8, mut out: u32, mut cur_pos: u8) -> (u32, u8) {
        while self.current_index < 8 && cur_pos < len {
            let va = (self.current_byte as u32 >> self.current_index) & 1;
            out += (va << cur_pos) as u32;
            cur_pos += 1;
            self.current_index += 1;
        }
        (out, cur_pos)
    }
    fn read_from_buf_u16(&mut self, len: u8, mut out: u16, mut cur_pos: u8) -> (u16, u8) {
        while self.current_index < 8 && cur_pos < len {
            let va = (self.current_byte as u16 >> self.current_index) & 1;
            out += (va << cur_pos) as u16;
            cur_pos += 1;
            self.current_index += 1;
        }
        (out, cur_pos)
    }



    /// Discard all the unread bits in the current byte and return a mutable reference
    /// to the underlying reader.
    pub fn borrow_reader_from_boundary(&mut self) -> &mut T {
        self.current_index = -1;
        &mut self.stream
    }
}


////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::ReadBytesExt;

    #[test]
    fn read_bits() -> io::Result<()> {
        let data: &[u8] = &[0b01100011, 0b11011011, 0b10101111];
        let mut reader = BitReader::new(data);
        assert_eq!(reader.read_bits(1)?, BitSequence::new(0b1, 1));
        assert_eq!(reader.read_bits(2)?, BitSequence::new(0b01, 2));
        assert_eq!(reader.read_bits(3)?, BitSequence::new(0b100, 3));
        assert_eq!(reader.read_bits(4)?, BitSequence::new(0b1101, 4));
        assert_eq!(reader.read_bits(5)?, BitSequence::new(0b10110, 5));
        assert_eq!(reader.read_bits(8)?, BitSequence::new(0b01011111, 8));
        assert_eq!(
            reader.read_bits(2).unwrap_err().kind(),
            io::ErrorKind::UnexpectedEof
        );
        Ok(())
    }

    #[test]
    fn borrow_reader_from_boundary() -> io::Result<()> {
        let data: &[u8] = &[0b01100011, 0b11011011, 0b10101111];
        let mut reader = BitReader::new(data);
        assert_eq!(reader.read_bits(3)?, BitSequence::new(0b011, 3));
        assert_eq!(reader.borrow_reader_from_boundary().read_u8()?, 0b11011011);
        assert_eq!(reader.read_bits(8)?, BitSequence::new(0b10101111, 8));
        Ok(())
    }
}
