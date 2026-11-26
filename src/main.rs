#![feature(uint_gather_scatter_bits)]

use color_eyre::eyre::{Ok, OptionExt, Result, eyre};
use regex::Regex;
use regex::regex;
use std::fs;
use std::sync::LazyLock;

static REGEX_PATTERN: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^G1 X(\d*\.\d*) Y(\d*\.\d*) E(\d*\.\d*)$").unwrap());

#[derive(Debug)]
struct Encoded(u8, u8, u8);
impl Encoded {
    /// Encode a byte into a triplet
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
fn encode_line(line: &str, data: &Encoded) -> Result<String> {
    //G1 X113.636 Y136.364 E.80952
    let caps = &REGEX_PATTERN
        .captures(line.trim())
        .ok_or_eyre(eyre!("Group did not match in line"))?;
    let x = &caps[1];
    let y = &caps[2];
    let e = &caps[3];
    Ok(format!(
        "G1 X{}{} Y{}{} E{}{}",
        x, data.0, y, data.1, e, data.2
    ))
}
#[allow(clippy::cast_possible_truncation)]
fn decode_line(line: &str) -> Result<u8> {
    let caps = &REGEX_PATTERN
        .captures(line.trim())
        .ok_or_eyre(eyre!("Group did not match in line"))?;
    let x = caps[1]
        .chars()
        .last()
        .ok_or_eyre(eyre!("Could not get last character of X"))?
        .to_digit(8)
        .ok_or_eyre(eyre!("Could not convert char to uint"))? as u8;
    let y = caps[2]
        .chars()
        .last()
        .ok_or_eyre(eyre!("Could not get last character of Y"))?
        .to_digit(8)
        .ok_or_eyre(eyre!("Could not convert char to uint"))? as u8;
    let e = caps[3]
        .chars()
        .last()
        .ok_or_eyre(eyre!("Could not get last character of E"))?
        .to_digit(8)
        .ok_or_eyre(eyre!("Could not convert char to uint"))? as u8;
    let enc = Encoded(x, y, e);
    Ok(enc.decode_byte()?)
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

#[test]
fn test_encode_line() {
    let initial_line = "G1 X113.636 Y136.364 E.80952";
    let expected = "G1 X113.6365 Y136.3643 E.809526".to_string();
    let data_to_encode = Encoded::encode_byte(0xAF).unwrap();
    let line_encoded = encode_line(initial_line, &data_to_encode).unwrap();
    assert_eq!(expected, line_encoded);
}
#[test]
fn test_decode_line() {
    let line_encoded = "G1 X113.6365 Y136.3643 E.809526".to_string();
    let expected_data = 0xAF;
    let decoded_data = decode_line(&line_encoded).unwrap();
    assert_eq!(expected_data, decoded_data);
}
