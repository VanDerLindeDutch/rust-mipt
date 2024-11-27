#![forbid(unsafe_code)]

use std::{collections::HashMap, convert::TryFrom, io::BufRead};

use anyhow::{bail, Context, Result};

use crate::bit_reader::{BitReader, BitSequence};
use crate::deflate::huffman_coding::LitLenToken::{EndOfBlock, Length, Literal};

pub(super) fn decode_litlen_distance_trees<T: BufRead>(
    bit_reader: &mut BitReader<T>,
) -> Result<(HuffmanCoding<LitLenToken>, HuffmanCoding<DistanceToken>)> {
    let hlit = bit_reader.read_bits(5)?.bits();
    let hdist = bit_reader.read_bits(5)?.bits();
    let hclen = bit_reader.read_bits(4)?.bits();
    let mut codeLen = vec![0u8; 19];
    for i in 16..=18 {
        codeLen[i] = bit_reader.read_bits(3)?.bits() as u8;
    }
    codeLen[0] = bit_reader.read_bits(3)?.bits() as u8;
    for i in 0..hclen {
        let i = i as usize;
        match i % 2 {
            0 => {
                codeLen[8 + i / 2] = bit_reader.read_bits(3)?.bits() as u8;
            }
            1 => {
                codeLen[7 - i / 2] = bit_reader.read_bits(3)?.bits() as u8;
            }
            _ => { unreachable!() }
        }
    }
    let code_len_coding = HuffmanCoding::<TreeCodeToken>::from_lengths(codeLen.as_slice())?;
    let mut lit_len_lens = Vec::with_capacity((hlit + hdist + 258) as usize);
    while lit_len_lens.len() < (hlit + hdist + 258) as usize {
        let sym = code_len_coding.read_symbol(bit_reader);
        if sym.is_err() {
            println!("{}", lit_len_lens.len())
        }
        let sym = sym?;
        match sym {
            TreeCodeToken::Length(v) => {
                lit_len_lens.push(v);
            }
            TreeCodeToken::CopyPrev => {
                let to_copy = bit_reader.read_bits(2)?.bits();
                let val = *lit_len_lens.last().unwrap();
                for _ in 0..to_copy + 3 {
                    lit_len_lens.push(val);
                }
            }
            TreeCodeToken::RepeatZero { base, extra_bits } => {
                let extra = bit_reader.read_bits(extra_bits)?.bits();
                for _ in 0..base + extra {
                    lit_len_lens.push(0);
                }
            }
        }
    }
    let hlit = hlit as usize;
    let mut dist_vec = Vec::from(&lit_len_lens[hlit + 257..]);
    if dist_vec.iter().filter(|x| **x >= 1).count() == 1 {
        while dist_vec.len() < 32 {
            dist_vec.push(0);
        }
        dist_vec[31] = 1;
    }
    Ok((HuffmanCoding::<LitLenToken>::from_lengths(&lit_len_lens[0..hlit + 257])?,
        HuffmanCoding::<DistanceToken>::from_lengths(&dist_vec)?))


    // See RFC 1951, section 3.2.7.
    // TODO: your code goes here.
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug)]
enum TreeCodeToken {
    Length(u8),
    CopyPrev,
    RepeatZero { base: u16, extra_bits: u8 },
}

impl TryFrom<HuffmanCodeWord> for TreeCodeToken {
    type Error = anyhow::Error;

    fn try_from(value: HuffmanCodeWord) -> Result<Self> {
        match value.0 {
            0..=15 => {
                Ok(Self::Length(value.0 as u8))
            }
            16 => {
                Ok(Self::CopyPrev)
            }
            17 => {
                Ok(Self::RepeatZero { base: 3, extra_bits: 3 })
            }
            18 => {
                Ok(Self::RepeatZero { base: 11, extra_bits: 7 })
            }
            _ => { bail!("incorrect treecode token") }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug)]
pub enum LitLenToken {
    Literal(u8),
    EndOfBlock,
    Length { base: u16, extra_bits: u8 },
}

impl TryFrom<HuffmanCodeWord> for LitLenToken {
    type Error = anyhow::Error;

    fn try_from(value: HuffmanCodeWord) -> Result<Self> {
        // See RFC 1951, section 3.2.5.
        // TODO: your code goes here.
        match value.0 {
            ..256 => {
                Ok(Literal(value.0 as u8))
            }
            256 => {
                Ok(EndOfBlock)
            }
            ..=264 => {
                Ok(Length { base: value.0 - 254, extra_bits: 0 })
            }
            285 => {
                Ok(Length { base: 258, extra_bits: 0 })
            }
            v @ 265..285 => {
                let extra_bits = (((v - 265) / 4) + 1) as u8;
                let base = match v {
                    265..=268 => {
                        let out = 11 + 2 * ((v - 265) % 4);
                        out
                    }
                    269..=272 => {
                        let out = 19 + 4 * ((v - 265) % 4);
                        out
                    }
                    273..=276 => {
                        let out = 35 + 8 * ((v - 265) % 4);
                        out
                    }
                    277..=280 => {
                        let out = 67 + 16 * ((v - 265) % 4);
                        out
                    }
                    281..=284 => {
                        let out = 131 + 32 * ((v - 265) % 4);
                        out
                    }
                    _ => unreachable!()
                };
                Ok(Length { base, extra_bits })
            }
            v => {
                bail!("max LitLenToken value is 285")
            }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug)]
pub struct DistanceToken {
    pub base: u16,
    pub extra_bits: u8,
}

impl TryFrom<HuffmanCodeWord> for DistanceToken {
    type Error = anyhow::Error;

    fn try_from(value: HuffmanCodeWord) -> Result<Self> {
        // See RFC 1951, section 3.2.5.
        // TODO: your code goes here.

        match value.0 {
            0..=3 => {
                Ok(Self { base: value.0 + 1, extra_bits: 0 })
            }
            4..=29 => {
                let extra_bits = (value.0 / 2) as u8 - 1;
                let out = Self { base: ((value.0 % 2 + 2) << extra_bits) + 1, extra_bits };
                Ok(out)
            }
            _ => { bail!("incorrect distance token") }
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

const MAX_BITS: usize = 15;

pub struct HuffmanCodeWord(pub u16);

pub struct HuffmanCoding<T> {
    map: HashMap<BitSequence, T>,
}

impl<T> HuffmanCoding<T>
where
    T: Copy + TryFrom<HuffmanCodeWord, Error=anyhow::Error>,
{
    pub fn new(map: HashMap<BitSequence, T>) -> Self {
        Self { map }
    }

    #[allow(unused)]
    pub fn decode_symbol(&self, seq: BitSequence) -> Option<T> {
        self.map.get(&seq).copied()
    }

    pub fn read_symbol<U: BufRead>(&self, bit_reader: &mut BitReader<U>) -> Result<T> {
        let mut r = BitSequence::new(0, 0);
        loop {
            let b = bit_reader.read_bits(1)?;
            r = r.concat(b);
            if r.len() > MAX_BITS as u8 {
                bail!("incorrect huffman code")
            }
            if let Some(o) = self.map.get(&r) {
                return Ok(o.clone());
            }
        }
    }

    pub fn from_lengths(code_lengths: &[u8]) -> Result<Self> {
        let mut map = HashMap::<BitSequence, T>::new();
        let mut bl_count = [0; MAX_BITS + 1];
        for x in code_lengths {
            bl_count[*x as usize] += 1;
        }
        let mut code = 0;
        let mut next_code: [u16; MAX_BITS + 1] = [0; MAX_BITS + 1];
        bl_count[0] = 0;
        for bits in 1..=MAX_BITS {
            code = (code + bl_count[bits - 1]) << 1;
            next_code[bits] = code;
        }
        for x in code_lengths.iter().enumerate() {
            if *x.1 == 0 {
                continue;
            }
            if x.0 > u16::MAX as usize {
                bail!("max value is 2^16-1")
            }
            map.insert(BitSequence::new(next_code[*x.1 as usize], *x.1), T::try_from(HuffmanCodeWord(x.0 as u16))?);
            next_code[*x.1 as usize] += 1;
        }
        // See RFC 1951, section 3.2.2.
        // TODO: your code goes here.
        Ok(Self { map })
    }
    // 00
}

////////////////////////////////////////////////////////////////////////////////

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Copy, Debug, PartialEq)]
    struct Value(u16);

    impl TryFrom<HuffmanCodeWord> for Value {
        type Error = anyhow::Error;

        fn try_from(x: HuffmanCodeWord) -> Result<Self> {
            Ok(Self(x.0))
        }
    }

    #[test]
    fn from_lengths() -> Result<()> {
        let code = HuffmanCoding::from_lengths(&[2, 3, 4, 3, 3, 4, 2])?;

        assert_eq!(
            code.decode_symbol(BitSequence::new(0b00, 2)),
            Some(Value(0)),
        );
        assert_eq!(
            code.decode_symbol(BitSequence::new(0b100, 3)),
            Some(Value(1)),
        );
        assert_eq!(
            code.decode_symbol(BitSequence::new(0b1110, 4)),
            Some(Value(2)),
        );
        assert_eq!(
            code.decode_symbol(BitSequence::new(0b101, 3)),
            Some(Value(3)),
        );
        assert_eq!(
            code.decode_symbol(BitSequence::new(0b110, 3)),
            Some(Value(4)),
        );
        assert_eq!(
            code.decode_symbol(BitSequence::new(0b1111, 4)),
            Some(Value(5)),
        );
        assert_eq!(
            code.decode_symbol(BitSequence::new(0b01, 2)),
            Some(Value(6)),
        );

        assert_eq!(code.decode_symbol(BitSequence::new(0b0, 1)), None);
        assert_eq!(code.decode_symbol(BitSequence::new(0b10, 2)), None);
        assert_eq!(code.decode_symbol(BitSequence::new(0b111, 3)), None,);

        Ok(())
    }

    #[test]
    fn read_symbol() -> Result<()> {
        let code = HuffmanCoding::<Value>::from_lengths(&[2, 3, 4, 3, 3, 4, 2])?;
        let mut data: &[u8] = &[0b10111001, 0b11001010, 0b11101101];
        let mut reader = BitReader::new(&mut data);

        assert_eq!(code.read_symbol(&mut reader)?, Value(1));
        assert_eq!(code.read_symbol(&mut reader)?, Value(2));
        assert_eq!(code.read_symbol(&mut reader)?, Value(3));
        assert_eq!(code.read_symbol(&mut reader)?, Value(6));
        assert_eq!(code.read_symbol(&mut reader)?, Value(0));
        assert_eq!(code.read_symbol(&mut reader)?, Value(2));
        assert_eq!(code.read_symbol(&mut reader)?, Value(4));
        assert!(code.read_symbol(&mut reader).is_err());

        Ok(())
    }

    #[test]
    fn from_lengths_with_zeros() -> Result<()> {
        let lengths = [3, 4, 5, 5, 0, 0, 6, 6, 4, 0, 6, 0, 7];
        let code = HuffmanCoding::<Value>::from_lengths(&lengths)?;
        let mut data: &[u8] = &[
            0b00100000, 0b00100001, 0b00010101, 0b10010101, 0b00110101, 0b00011101,
        ];
        let mut reader = BitReader::new(&mut data);

        assert_eq!(code.read_symbol(&mut reader)?, Value(0));
        assert_eq!(code.read_symbol(&mut reader)?, Value(1));
        assert_eq!(code.read_symbol(&mut reader)?, Value(2));
        assert_eq!(code.read_symbol(&mut reader)?, Value(3));
        assert_eq!(code.read_symbol(&mut reader)?, Value(6));
        assert_eq!(code.read_symbol(&mut reader)?, Value(7));
        assert_eq!(code.read_symbol(&mut reader)?, Value(8));
        assert_eq!(code.read_symbol(&mut reader)?, Value(10));
        assert_eq!(code.read_symbol(&mut reader)?, Value(12));
        assert!(code.read_symbol(&mut reader).is_err());

        Ok(())
    }

    #[test]
    fn from_lengths_additional() -> Result<()> {
        let lengths = [
            9, 10, 10, 8, 8, 8, 5, 6, 4, 5, 4, 5, 4, 5, 4, 4, 5, 4, 4, 5, 4, 5, 4, 5, 5, 5, 4, 6, 6,
        ];
        let code = HuffmanCoding::<Value>::from_lengths(&lengths)?;
        let mut data: &[u8] = &[
            0b11111000, 0b10111100, 0b01010001, 0b11111111, 0b00110101, 0b11111001, 0b11011111,
            0b11100001, 0b01110111, 0b10011111, 0b10111111, 0b00110100, 0b10111010, 0b11111111,
            0b11111101, 0b10010100, 0b11001110, 0b01000011, 0b11100111, 0b00000010,
        ];
        let mut reader = BitReader::new(&mut data);

        assert_eq!(code.read_symbol(&mut reader)?, Value(10));
        assert_eq!(code.read_symbol(&mut reader)?, Value(7));
        assert_eq!(code.read_symbol(&mut reader)?, Value(27));
        assert_eq!(code.read_symbol(&mut reader)?, Value(22));
        assert_eq!(code.read_symbol(&mut reader)?, Value(9));
        assert_eq!(code.read_symbol(&mut reader)?, Value(0));
        assert_eq!(code.read_symbol(&mut reader)?, Value(11));
        assert_eq!(code.read_symbol(&mut reader)?, Value(15));
        assert_eq!(code.read_symbol(&mut reader)?, Value(2));
        assert_eq!(code.read_symbol(&mut reader)?, Value(20));
        assert_eq!(code.read_symbol(&mut reader)?, Value(8));
        assert_eq!(code.read_symbol(&mut reader)?, Value(4));
        assert_eq!(code.read_symbol(&mut reader)?, Value(23));
        assert_eq!(code.read_symbol(&mut reader)?, Value(24));
        assert_eq!(code.read_symbol(&mut reader)?, Value(5));
        assert_eq!(code.read_symbol(&mut reader)?, Value(26));
        assert_eq!(code.read_symbol(&mut reader)?, Value(18));
        assert_eq!(code.read_symbol(&mut reader)?, Value(12));
        assert_eq!(code.read_symbol(&mut reader)?, Value(25));
        assert_eq!(code.read_symbol(&mut reader)?, Value(1));
        assert_eq!(code.read_symbol(&mut reader)?, Value(3));
        assert_eq!(code.read_symbol(&mut reader)?, Value(6));
        assert_eq!(code.read_symbol(&mut reader)?, Value(13));
        assert_eq!(code.read_symbol(&mut reader)?, Value(14));
        assert_eq!(code.read_symbol(&mut reader)?, Value(16));
        assert_eq!(code.read_symbol(&mut reader)?, Value(17));
        assert_eq!(code.read_symbol(&mut reader)?, Value(19));
        assert_eq!(code.read_symbol(&mut reader)?, Value(21));

        Ok(())
    }
}
