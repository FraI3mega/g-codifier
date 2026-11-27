use color_eyre::eyre::{Ok, OptionExt, Result, eyre};
use regex::{Captures, Regex};
use std::sync::LazyLock;

pub static REGEX_PATTERN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^G1(.*)X(-?\d*\.\d*)(.*)Y(-?\d*\.\d*)(.*)E(-?\d*\.\d*)(.*)$").unwrap()
});

#[derive(Debug, PartialEq)]
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
pub fn encode_line(line: &str, data: u8) -> Result<String> {
    //G1 X113.636 Y136.364 E.80952
    let data_encoded = Encoded::encode_byte(data)?;
    let x = data_encoded.0;
    let y = data_encoded.1;
    let e = data_encoded.2;
    let string_rep = REGEX_PATTERN
        .replace_all(line, |caps: &Captures| {
            format!(
                "G1{}X{}{x}{}Y{}{y}{}E{}{e}{}",
                &caps[1], &caps[2], &caps[3], &caps[4], &caps[5], &caps[6], &caps[7]
            )
        });
        match string_rep {
            std::borrow::Cow::Borrowed(_) => Err(eyre!("Failed to encode line")),
            std::borrow::Cow::Owned(string) => Ok(string),
        }
}
#[allow(clippy::cast_possible_truncation)]
pub fn decode_line(line: &str) -> Result<u8> {
    let caps = &REGEX_PATTERN
        .captures(line.trim())
        .ok_or_eyre(eyre!("Group did not match in line"))?;
    let x = caps[2]
        .chars()
        .last()
        .ok_or_eyre(eyre!("Could not get last character of X"))?
        .to_digit(8)
        .ok_or_eyre(eyre!("Could not convert char to uint"))? as u8;
    let y = caps[4]
        .chars()
        .last()
        .ok_or_eyre(eyre!("Could not get last character of Y"))?
        .to_digit(8)
        .ok_or_eyre(eyre!("Could not convert char to uint"))? as u8;
    let e = caps[6]
        .chars()
        .last()
        .ok_or_eyre(eyre!("Could not get last character of E"))?
        .to_digit(8)
        .ok_or_eyre(eyre!("Could not convert char to uint"))? as u8;
    let enc = Encoded(x, y, e);
    Ok(enc.decode_byte()?)
}

#[test]
fn test_encode() {
    let byte: u8 = 0xAF;
    let encoded = Encoded::encode_byte(byte).unwrap();
    assert_eq!(Encoded(5, 3, 6), encoded);
}

#[test]
fn test_decode() {
    let encoded = Encoded(5, 3, 6);
    let decoded = encoded.decode_byte().unwrap();
    assert_eq!(0xAF, decoded);
}

#[test]
fn test_encode_decode() {
    let byte: u8 = 0xAF;
    let encoded = Encoded::encode_byte(byte).unwrap();
    let decoded = encoded.decode_byte().unwrap();
    assert_eq!(byte, decoded);
}

#[test]
fn test_decode_bad_parity() {
    let mut enc = Encoded::encode_byte(0xAF).unwrap();
    enc.0 ^= 0b010;
    let dec = enc.decode_byte();
    assert!(dec.is_err());
}

#[test]
fn test_encode_line() {
    let initial_line = "G1 X113.636 Y136.364 E-.80952; test";
    let expected = "G1 X113.6365 Y136.3643 E-.809526; test".to_string();
    let line_encoded = encode_line(initial_line, 0xAF).unwrap();
    assert_eq!(expected, line_encoded);
}

#[test]
fn test_encode_line_no_matching_group() {
    let initial_line = "G1 X113.636 Y136.364";
    let line_encoded = encode_line(initial_line, 0xAF);
    dbg!(&line_encoded);
    assert!(line_encoded.is_err());
}
#[test]
fn test_decode_line() {
    let line_encoded = "G1 X113.6365 Y136.3643 E.809526".to_string();
    let expected_data = 0xAF;
    let decoded_data = decode_line(&line_encoded).unwrap();
    assert_eq!(expected_data, decoded_data);
}
#[test]
fn test_decode_line_no_matching_group() {
    let initial_line = "G1 X113.636 Y136.364";
    let line_decoded = decode_line(initial_line);
    assert!(line_decoded.is_err());
}

#[test]
fn test_decode_line_no_last_char_invalid_digit() {
    let initial_line = "G1 X113.636 Y136.364 E.8958";
    let line_decoded = decode_line(initial_line);
    assert!(line_decoded.is_err());
}

#[test]
fn test_encode_decode_line() {
    let line = "G1 X113.636 Y136.364 E.80952";
    let data = 0xAF_u8;
    let encoded_line = encode_line(line, data).unwrap();
    let decoded_data = decode_line(&encoded_line).unwrap();
    assert_eq!(data, decoded_data);
}

#[test]
fn test_regex_matching() {
    let test_cases_pos = [
        "G1 X10.500 Y20.250 E0.04500",
        "G1 F1800 X10.0 Y20.0 Z0.20 E1.50000 ; move with comment",
        "G1 X50.000 Y60.000 E-0.75000",
        "G1 X.5 Y.25 E-.1",
        "G1 X12.34 Y56.78 E0.12345 ; X99.9 Y99.9 E9.9",
    ];
    let result = test_cases_pos
        .into_iter()
        .all(|x| REGEX_PATTERN.is_match(x));
    assert!(result);
}
