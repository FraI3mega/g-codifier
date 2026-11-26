#![feature(uint_gather_scatter_bits)]
#![feature(file_buffered)]

use clap::Parser;
use clap::Subcommand;
use color_eyre::eyre::{Ok, OptionExt, Result, eyre};
use regex::Regex;
use std::fs;
use std::fs::File;
use std::io::BufRead;
use std::sync::LazyLock;

use crate::encoded::{decode_line, encode_line};

mod encoded;

static REGEX_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^G1 X(\d*\.\d*) Y(\d*\.\d*) E(\d*\.\d*)$").unwrap());

#[derive(Debug)]
#[derive(Subcommand)]
enum Command {
    ///Encode the data file into input
    Encode {
        /// Input g-code file
        input: String,
        /// File to hide
        data: String,
        /// Processed g-code file
        #[arg(default_value_t = "out.gcode".to_string())]
        output: String,
    },
    ///Decode the data from input
    Decode {
        /// Input g-code file with hidden data
        input: String,
        /// Extracted data
        data: String,
        /// Original g-code file
        #[arg(default_value_t = "out.gcode".to_string())]
        output: String,
    },
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args = Args::parse();

    match args.command {
        Command::Encode { input, data, output } => {
            let input_file = File::open_buffered(input)?;
            let data_file = File::open_buffered(data)?;
            let mut output_file = File::create(output)?;

            
        },
        Command::Decode { input, data, output } => {
            let input_file = File::open_buffered(input)?;
            let data_file = File::open_buffered(data)?;
            let mut output_file = File::create(output)?;
            todo!("Decode")
        }
    }

    Ok(())
}
