#![feature(uint_gather_scatter_bits)]

use color_eyre::eyre::{Ok, OptionExt, Result, eyre};
use regex::Regex;
use std::fs;
fn main() -> Result<()> {
    color_eyre::install()?;
    println!("Hello, world!");

    let re = Regex::new(r"^G1 X\d*\.(\d*) Y\d*\.(\d*) E\d*\.(\d*)$")?;

    let mut file = fs::read_to_string("test-reduced.gcode")?;
    println!("{}", file.lines().filter(|line| re.is_match(line)).count());
    for line in file.lines().filter(|line| re.is_match(line)) {
        println!("{line}");
    }
    Ok(())
}

fn encode_byte(byte: u8) -> Result<[u8; 3]> {
    let parity: u8 = (byte.count_ones() % 2).try_into()?;
    let d1 = byte.extract_bits(0b1110_0000);
    let d2 = byte.extract_bits(0b0001_1100);
    let d3 = byte.extract_bits(0b0000_0011) << 1 | parity;
    Ok([d1, d2, d3])
}

/// Decodes a triplet into a byte
fn decode_byte(input: [u8; 3]) -> Result<u8> {
    let (d1, d2, d3) = input.into();
    let byte = d1 << 5 | d2 << 2 | d3.extract_bits(0b0000_0110);
    if byte.count_ones() % 2 == u32::from(d3 & 0b0000_0001) {
        Ok(byte)
    } else {
        Err(eyre!("Parity invalid"))
    }
}

#[test]
fn test_encode_decode() {
    let byte: u8 = 0xAF;
    let encoded = encode_byte(byte).unwrap();
    let decoded = decode_byte(encoded).unwrap();
    assert_eq!(byte, decoded);
}
