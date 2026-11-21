#![feature(uint_gather_scatter_bits)]

use color_eyre::eyre::{Ok, OptionExt, Result, eyre};
use regex::Regex;
use std::fs;

struct Encoded(u8, u8, u8);
impl Encoded {
    fn encode_byte(byte: u8) -> Result<Self> {
        let parity: u8 = (byte.count_ones() % 2).try_into()?;
        let d1 = byte.extract_bits(0b1110_0000);
        let d2 = byte.extract_bits(0b0001_1100);
        let d3 = byte.extract_bits(0b0000_0011) << 1 | parity;
        Ok(Self(d1, d2, d3))
    }

    /// Decodes a triplet into a byte
    fn decode_byte(self) -> Result<u8> {
        let byte = self.0 << 5 | self.1 << 2 | self.2.extract_bits(0b0000_0110);
        if byte.count_ones() % 2 == u32::from(self.2 & 0b0000_0001) {
            Ok(byte)
        } else {
            Err(eyre!("Parity invalid"))
        }
    }
}
fn main() -> Result<()> {
    color_eyre::install()?;
    println!("Hello, world!");

    let re = Regex::new(r"^G1 X\d*\.(\d*) Y\d*\.(\d*) E\d*\.(\d*)$")?;

    let mut file = fs::read_to_string("test-reduced.gcode")?;
    let data_to_hide = fs::read("data-to-hide")?;
    println!("{}", file.lines().filter(|line| re.is_match(line)).count());
    for line in file.lines().filter(|line| re.is_match(line)) {
        println!("{line}");
    }
    Ok(())
}

#[test]
fn test_encode_decode() {
    let byte: u8 = 0xAF;
    let encoded = Encoded::encode_byte(byte).unwrap();
    let decoded = encoded.decode_byte().unwrap();
    assert_eq!(byte, decoded);
}
