

use std::fmt::Write;
use std::io::BufRead;
use anyhow::bail;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt};
use chrono::DateTime;
use crc::Crc;
use log::info;
use crate::gzip::{CM_DEFLATE, ID1, ID2};
use crate::gzip::flags::MemberFlags;

#[derive(Debug)]
pub struct MemberHeader {
    pub compression_method: CompressionMethod,
    pub modification_time: u32,
    pub extra: Option<Vec<u8>>,
    pub name: Option<String>,
    pub comment: Option<String>,
    pub extra_flags: u8,
    pub os: u8,
    pub has_crc: bool,
    pub is_text: bool,
}

impl MemberHeader {
    pub fn crc16(&self) -> u16 {
        let crc = Crc::<u32>::new(&crc::CRC_32_ISO_HDLC);
        let mut digest = crc.digest();

        digest.update(&[ID1, ID2, self.compression_method.into(), self.flags().0]);
        digest.update(&self.modification_time.to_le_bytes());
        digest.update(&[self.extra_flags, self.os]);

        if let Some(extra) = &self.extra {
            digest.update(&(extra.len() as u16).to_le_bytes());
            digest.update(extra);
        }

        if let Some(name) = &self.name {
            digest.update(name.as_bytes());
            digest.update(&[0]);
        }

        if let Some(comment) = &self.comment {
            digest.update(comment.as_bytes());
            digest.update(&[0]);
        }

        ((digest.finalize()) & 0xffff) as u16
    }

    pub fn flags(&self) -> MemberFlags {
        let mut flags = MemberFlags(0);
        flags.set_is_text(self.is_text);
        flags.set_has_crc(self.has_crc);
        flags.set_has_extra(self.extra.is_some());
        flags.set_has_name(self.name.is_some());
        flags.set_has_comment(self.comment.is_some());
        flags
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CompressionMethod {
    Deflate,
    Unknown(u8),
}

impl From<u8> for CompressionMethod {
    fn from(value: u8) -> Self {
        match value {
            CM_DEFLATE => Self::Deflate,
            x => Self::Unknown(x),
        }
    }
}

impl From<CompressionMethod> for u8 {
    fn from(method: CompressionMethod) -> u8 {
        match method {
            CompressionMethod::Deflate => CM_DEFLATE,
            CompressionMethod::Unknown(x) => x,
        }
    }
}


#[derive(Debug)]
pub struct MemberFooter {
    pub data_crc32: u32,
    pub data_size: u32,
}

impl<T: BufRead, I: std::io::Write> crate::gzip::GzipReader<T, I> {
   pub(super) fn parse_header(&mut self) -> anyhow::Result<(MemberHeader, MemberFlags)> {
        let mut reader = &mut self.reader;
        let id1 = reader.read_bits(8)?;
        if id1.bits() != 0x1f {
            bail!("wrong id values")
        }
        let id2 = reader.read_bits(8)?;
        if id2.bits() != 0x8b {
            bail!("wrong id values")
        }

        let cm = reader.read_bits(8)?;
        let ftext = reader.read_bits(1)?;
        let fhcrc = reader.read_bits(1)?;
        let fextra = reader.read_bits(1)?;
        let fname = reader.read_bits(1)?;
        let fcomment = reader.read_bits(1)?;
        let reserved_bits = reader.read_bits(3)?;

        if reserved_bits.bits() != 0 {
            bail!("reserved bits must be zeroes")
        }
        let mtime = reader.read_u32()?;
        let date = DateTime::from_timestamp(mtime as i64, 0).unwrap();
       let q = date.to_string();


        let xf = reader.read_bits(8)?;
        match xf.bits() {
            2 => info!("maximum compression"),
            4 => info!("fastest compression"),
            _ => {}
        }
        let os = reader.read_bits(8)?;

        let mut out = MemberHeader {
            compression_method: CompressionMethod::from(cm.bits() as u8),
            modification_time: mtime,
            extra: None,
            name: None,
            comment: None,
            extra_flags: 0,
            os: os.bits() as u8,
            has_crc: false,
            is_text: false,
        };
        if ftext.bits() == 1 {
            out.is_text = true;
        }

        if fextra.bits() == 1 {
            let len = reader.borrow_reader_from_boundary().read_u16::<LittleEndian>()?;
            let mut extra = Vec::with_capacity(len as usize);
            for _ in 0..len {
                extra.push(reader.read_bits(8)?.bits() as u8);
            }
            out.extra = Some(extra);
        }
        if fname.bits() == 1 {
            let fname = reader.read_str_to_null()?;
            info!("file name is {}", fname);
            out.name = Some(fname);
        }
        if fcomment.bits() == 1 {
            let comment = reader.read_str_to_null()?;
            info!("comment is {}", comment);
            out.comment = Some(comment);
        }
        if fhcrc.bits() == 1 {
            out.has_crc = true;
            let crc_16 = reader.borrow_reader_from_boundary().read_u16::<LittleEndian>()?;
            if crc_16 != out.crc16() {
                bail!("header crc16 check failed")
            }
            info!("crc 16 is {}", crc_16);



        }

        // See RFC 1952, section 2.3.
        // TODO: your code goes here.
        let flags = out.flags();
        Ok((out, flags))
    }

    pub(super) fn parse_footer(&mut self, length: usize, crc_32: u32) -> anyhow::Result<()> {
        let mut reader = &mut self.reader;
        let crc_from_footer =  reader.borrow_reader_from_boundary().read_u32::<LittleEndian>()?;
        if crc_from_footer != crc_32 {
            bail!("crc32 check failed")
        }
        let length_from_footer = reader.borrow_reader_from_boundary().read_u32::<LittleEndian>()?;
        if length_from_footer as usize != length {
            bail!("length check failed")
        }
        Ok(())
    }
}

