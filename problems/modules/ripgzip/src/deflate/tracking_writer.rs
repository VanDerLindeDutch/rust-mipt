#![forbid(unsafe_code)]

use std::io::{self, BufWriter, Write};

use anyhow::{bail, Result};
use crc::CRC_32_ISO_HDLC;
////////////////////////////////////////////////////////////////////////////////

const HISTORY_SIZE: usize = 32768;
const X25: crc::Crc<u32> = crc::Crc::<u32>::new(&CRC_32_ISO_HDLC);
pub struct TrackingWriter<T: Write> {
    inner: BufWriter<T>,
    buf: [u8; HISTORY_SIZE],
    current_index: usize,
    buf_len: usize,
    // TODO: your code goes here.
}

impl<T: Write> Write for TrackingWriter<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self.inner.write(buf) {
            Ok(v) => {
                for i in 0..v {
                    self.append_to_buf(buf[i]);
                }

                Ok(v)
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}

impl<T: Write> TrackingWriter<T> {
    pub fn new(inner: T) -> Self {
        Self {
            inner: BufWriter::new(inner),
            buf: [0; HISTORY_SIZE],
            current_index: 0,
            buf_len: 0,
        }
    }

    /// Write a sequence of `len` bytes written `dist` bytes ago.
    pub fn write_previous(&mut self, dist: usize, len: usize) -> Result<()> {
        if dist > self.buf_len {
            bail!("big distance");
        }
        let mut vec_to_write = Vec::with_capacity(len);
        let start_index = ((self.current_index as i64 - dist as i64 + HISTORY_SIZE as i64) % HISTORY_SIZE as i64) as usize;

        for i in 0..len {
            let i = (start_index+ i ) % HISTORY_SIZE;
            let b = self.buf[i % HISTORY_SIZE];
            vec_to_write.push(b);
            self.append_to_buf(b);
        }


        match self.inner.write(&vec_to_write) {
            Ok(v) => {
                if v != len {
                    bail!("previous written less than must");
                }
                Ok(())
            }
            Err(err) => {
                Err(anyhow::Error::from(err))
            }
        }
    }

    pub fn byte_count(&self) -> usize {
        self.buf_len
    }

    pub fn crc32(mut self) -> u32 {
        X25.checksum(&self.buf[0..self.current_index])
    }

    fn append_to_buf(&mut self, b: u8) {
        self.buf[self.current_index] = b;
        self.current_index += 1;
        if self.buf_len != HISTORY_SIZE {
            self.buf_len += 1;
        }
        if self.current_index == HISTORY_SIZE {
            self.current_index = 0;
            self.buf_len = HISTORY_SIZE;
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;
    use byteorder::WriteBytesExt;

    #[test]
    fn write() -> Result<()> {
        let mut buf: &mut [u8] = &mut [0u8; 10];
        let mut writer = TrackingWriter::new(&mut buf);

        assert_eq!(writer.write(&[1, 2, 3, 4])?, 4);
        assert_eq!(writer.byte_count(), 4);

        assert_eq!(writer.write(&[4, 8, 15, 16, 23])?, 5);
        assert_eq!(writer.byte_count(), 9);

        assert_eq!(writer.write(&[0, 0, 123])?, 1);
        assert_eq!(writer.byte_count(), 10);

        assert_eq!(writer.write(&[42, 124, 234, 27])?, 0);
        assert_eq!(writer.byte_count(), 10);
        assert_eq!(writer.crc32(), 2992191065);

        Ok(())
    }

    #[test]
    fn write_previous() -> Result<()> {
        let mut buf: &mut [u8] = &mut [0u8; 512];
        let mut writer = TrackingWriter::new(&mut buf);

        for i in 0..=255 {
            writer.write_u8(i)?;
        }

        writer.write_previous(192, 128)?;
        assert_eq!(writer.byte_count(), 384);

        assert!(writer.write_previous(10000, 20).is_err());
        assert_eq!(writer.byte_count(), 384);

        assert!(writer.write_previous(256, 256).is_err());
        assert_eq!(writer.byte_count(), 512);

        assert!(writer.write_previous(1, 1).is_err());
        assert_eq!(writer.byte_count(), 512);
        assert_eq!(writer.crc32(), 2733545866);

        Ok(())
    }
}
