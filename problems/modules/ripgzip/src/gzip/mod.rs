#![forbid(unsafe_code)]

extern crate chrono;

mod header;

mod flags;

use crate::bit_reader::BitReader;
use crate::deflate::{DeflateReader};
use anyhow::Result;
use byteorder::ReadBytesExt;

use std::fmt::Write;
use std::io::BufRead;
use crate::gzip::header::CompressionMethod;

const ID1: u8 = 0x1f;
const ID2: u8 = 0x8b;

const CM_DEFLATE: u8 = 8;

const FTEXT_OFFSET: u8 = 0;
const FHCRC_OFFSET: u8 = 1;
const FEXTRA_OFFSET: u8 = 2;
const FNAME_OFFSET: u8 = 3;
const FCOMMENT_OFFSET: u8 = 4;

pub trait Decoder<T: BufRead, I: std::io::Write> {
    fn decode(&mut self, _: BitReader<T>, _: I) -> Result<()>;
}
pub struct GzipReader<T, I> {
    reader: BitReader<T>,
    decoder: Box<dyn Decoder<T, I>>,
}

impl<T: BufRead, I: std::io::Write> GzipReader<T, I> {
    pub fn new(reader: T) -> Self {
        let decoder = DeflateReader::new();
        Self { reader: BitReader::new(reader), decoder: Box::new(decoder) }
    }

    pub fn decode(mut self, writer: I) -> Result<()> {
        let header = self.parse_header()?;
        match header.0.compression_method {
            CompressionMethod::Deflate => { self.decoder.decode(self.reader, writer) }
            CompressionMethod::Unknown(v) => { unimplemented!() }
        }
    }

}




