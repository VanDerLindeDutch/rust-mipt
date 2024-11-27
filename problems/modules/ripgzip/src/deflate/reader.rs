use crate::bit_reader::BitReader;
use crate::deflate::huffman_coding::{decode_litlen_distance_trees, DistanceToken, HuffmanCoding, LitLenToken};
use crate::deflate::tracking_writer::TrackingWriter;
use crate::deflate::{BlockHeader, CompressionType};
use anyhow::bail;
use byteorder::{LittleEndian, ReadBytesExt, WriteBytesExt};
use std::io::{BufRead, Write};




pub struct DeflateReader {
    fixed_tree: Vec<u8>,
    distance_tree: Vec<u8>,
    // TODO: your code goes here.
}

impl DeflateReader {
    pub fn new() -> Self {
        let mut len_vec = Vec::with_capacity(287);
        for i in 0..=143 {
            len_vec.push(8);
        }
        for i in 144..=255 {
            len_vec.push(9);
        }
        for i in 256..=279 {
            len_vec.push(7);
        }
        for i in 280..=287 {
            len_vec.push(8);
        }
        let dist_vec = vec![5; 32];
        Self { fixed_tree: len_vec, distance_tree: dist_vec }
    }
    fn next_block<T: BufRead>(&mut self, mut bit_reader: &mut BitReader<T>) -> anyhow::Result<BlockHeader> {
        let is_final = match bit_reader.read_bits(1)?.bits() {
            0 => false,
            1 => true,
            _ => unreachable!()
        };
        let c_type = match bit_reader.read_bits(2)?.bits() {
            0 => CompressionType::Uncompressed,
            1 => CompressionType::FixedTree,
            2 => CompressionType::DynamicTree,
            3 => CompressionType::Reserved,
            _ => unreachable!()
        };
        Ok(BlockHeader { is_final, compression_type: c_type })
    }


    pub fn read_block<T: BufRead, I: Write>(&mut self, mut bit_reader: &mut BitReader<T>,
                                            mut writer: &mut TrackingWriter<I>, ) -> anyhow::Result<bool> {
        let header = self.next_block(bit_reader)?;
        match header.compression_type {
            CompressionType::Uncompressed => {
                let inner_input = bit_reader.borrow_reader_from_boundary();
                let (len, nlen) = (inner_input.read_u16::<LittleEndian>()?, inner_input.read_u16::<LittleEndian>()?);
                if len ^ 0xFFFF != nlen {
                    bail!("incorrect len")
                }
                for _ in 0..len {
                    writer.write_u8(bit_reader.read_bits(8)?.bits() as u8)?;
                }
                Ok(header.is_final)
            }
            CompressionType::FixedTree => {
                self.read_compressed(bit_reader, writer, (HuffmanCoding::from_lengths(&self.fixed_tree)?, HuffmanCoding::from_lengths(&self.distance_tree)?))?;
                Ok(header.is_final)
            }
            CompressionType::DynamicTree => {
                let decoded = decode_litlen_distance_trees(&mut bit_reader)?;
                self.read_compressed(bit_reader, writer, decoded)?;
                Ok(header.is_final)
            }
            CompressionType::Reserved => {
                bail!("reserved type")
            }
        }
    }

    fn read_compressed<T: BufRead, I: Write>(&mut self, mut bit_reader: &mut BitReader<T>,
                                             mut writer: &mut TrackingWriter<I>, (lit_len_tree, distance_tree): (HuffmanCoding<LitLenToken>, HuffmanCoding<DistanceToken>)) -> anyhow::Result<()> {
        let mut count = 0;
        loop {
            count += 1;
            let sym = lit_len_tree.read_symbol(&mut bit_reader)?;
            match sym {
                LitLenToken::Literal(v) => {
                    let q = v as char;
                    // println!("{}", q);
                    let res = writer.write_u8(v);
                    if res.is_err() {
                        return Err(anyhow::Error::from(res.unwrap_err()));
                    }
                }
                LitLenToken::EndOfBlock => {
                    return Ok(());
                }
                LitLenToken::Length { mut base, extra_bits } => {
                    let extra = bit_reader.read_bits(extra_bits)?.bits();
                    let length = base + extra;

                    let mut distance = distance_tree.read_symbol(&mut bit_reader)?;
                    let extra_distance = bit_reader.read_bits(distance.extra_bits);
                    let extra_distance = extra_distance?.bits();
                    distance.base += extra_distance;
                    writer.write_previous(distance.base as usize, length as usize)?;
                }
            }
        }
    }

    fn get_fixed_tree(&mut self) {}
}
