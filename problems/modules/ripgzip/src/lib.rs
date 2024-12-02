#![forbid(unsafe_code)]

use std::io::{BufRead, Write};

use crate::gzip::GzipReader;
use anyhow::{Context, Result};
use log::*;

mod deflate;
mod gzip;
mod bit_reader;
mod tracking_writer;

pub fn decompress<R: BufRead, W: Write>(input: R, mut output: W) -> Result<()> {
    let mut gz = GzipReader::new(input);
    gz.decode(output)
}
