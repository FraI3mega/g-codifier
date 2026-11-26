#![feature(uint_gather_scatter_bits)]

use color_eyre::eyre::{Ok, OptionExt, Result, eyre};
use regex::Regex;
use std::fs;
use std::sync::LazyLock;

use crate::encoded::{decode_line, encode_line};

mod encoded;

static REGEX_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^G1 X(\d*\.\d*) Y(\d*\.\d*) E(\d*\.\d*)$").unwrap());


fn main() -> Result<()> {
    color_eyre::install()?;
    println!("Hello, world!");

    Ok(())
}

