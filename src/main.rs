#![feature(uint_gather_scatter_bits)]
#![feature(file_buffered)]

use clap::Parser;
use clap::Subcommand;
use color_eyre::Section;
use color_eyre::eyre::{Ok, Result, eyre};
use lzma_rust2::Lzma2Options;
use lzma_rust2::Lzma2Writer;
use regex::Regex;
use std::fs::File;
use std::io;
use std::io::BufRead;
use std::io::Cursor;
use std::io::Write;
use std::sync::LazyLock;

use crate::encoded::REGEX_PATTERN;
use crate::encoded::encode_line;

mod encoded;

#[derive(Debug, Subcommand)]
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
    },
    ///Get the max (compressed) data size, that can fit in gcode
    Capacity {
        /// G-code file to check
        input: String,
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
        Command::Encode {
            input,
            data,
            output,
        } => {
            let input_file = File::open_buffered(input)?;
            let mut data_file = File::open_buffered(data)?;
            let mut output_file = File::create_buffered(output)?;

            println!(
                "Data size before compression: {}B",
                data_file.get_ref().metadata()?.len()
            );

            let data_buffer: Cursor<Vec<u8>> = Cursor::new(Vec::new());
            let mut writer = Lzma2Writer::new(data_buffer, Lzma2Options::default());
            io::copy(&mut data_file, &mut writer)?;

            let writer = writer.finish()?;

            #[allow(clippy::cast_possible_truncation)]
            let data_len = writer.get_ref().len() as u32;

            println!("Data size after compression: {data_len}B");
            let mut data_vec: Vec<u8> = Vec::with_capacity(data_len as usize + 5);
            data_vec.push(0xF3); // preamble
            data_vec.extend(data_len.to_le_bytes());
            data_vec.extend(writer.into_inner());
            let mut data_iter = data_vec.into_iter();
            for line in input_file.lines() {
                let line = line?;
                if REGEX_PATTERN.is_match(&line)
                    && let Some(data) = data_iter.next()
                {
                    let line = encode_line(&line, data)?;
                    writeln!(output_file, "{line}")?;
                    continue;
                }
                writeln!(output_file, "{line}")?;
            }
            output_file.flush()?;

            if data_iter.next().is_some() {
                return Err(eyre!("Failed to insert all data")
                    .suggestion("Your gcode file may be too short to contain all the data"));
            }
        }
        Command::Decode { input, data } => {
            let input_file = File::open_buffered(input)?;
            let data_file = File::create_buffered(data)?;
            todo!("Decode")
        }
        Command::Capacity { input } => {
            let input_file = File::open_buffered(input)?;
            let mut count = -5;
            for line in input_file.lines() {
                let line = line?;
                if REGEX_PATTERN.is_match(&line) {
                    count += 1;
                }
            }
            println!("Max capacity: {count} B");
        }
    }

    Ok(())
}
