#![forbid(unsafe_code)]

mod huffman_coding;

mod tracking_writer;

pub mod reader;

pub use reader::DeflateReader;
use crate::gzip;
use anyhow::Result;
use crate::bit_reader::BitReader;
use byteorder::{ReadBytesExt, WriteBytesExt};
use std::{
    convert::TryFrom,
    io::{BufRead, Write},
};
use tracking_writer::TrackingWriter;

#[derive(Debug)]
struct BlockHeader {
    pub is_final: bool,
    pub compression_type: CompressionType,
}


#[derive(Debug)]
enum CompressionType {
    Uncompressed = 0,
    FixedTree = 1,
    DynamicTree = 2,
    Reserved = 3,
}




impl<T: BufRead, I: Write> gzip::Decoder<T, I> for DeflateReader {
    fn decode(&mut self, mut reader: BitReader<T>,
              writer: I) -> Result<()> {
        let mut tracking_writer = TrackingWriter::new(writer);
        while !self.read_block(&mut reader, &mut tracking_writer)? {}
        tracking_writer.flush()?;
        Ok(())
    }
}


