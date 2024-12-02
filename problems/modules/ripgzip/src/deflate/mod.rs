#![forbid(unsafe_code)]

mod huffman_coding;

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
use crate::tracking_writer::TrackingWriter;

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
    fn decode(&mut self, reader: &mut BitReader<T>,
              tracking_writer: &mut TrackingWriter<I>) -> Result<()> {

        while !self.read_block(reader, tracking_writer)? {}
        Ok(())
    }
}


