#![feature(uint_gather_scatter_bits)]
#![feature(file_buffered)]

use clap::{Parser, Subcommand};
use color_eyre::{
    Section,
    eyre::{Ok, Result, eyre},
};
use lzma_rust2::{Lzma2Options, Lzma2Reader, Lzma2Writer, LzmaOptions};
use std::io::{BufRead, Cursor, Write};
use std::{fs::File, io};

use crate::encoded::REGEX_PATTERN;
use crate::encoded::decode_line;
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
            let mut data_file = File::create_buffered(data)?;

            let mut lines = input_file.lines();

            //sig check
            for line in &mut lines {
                let line = line?;
                if REGEX_PATTERN.is_match(&line) {
                    if decode_line(&line)? == 0xF3 {
                        break;
                    }
                    return Err(eyre!("Invalid signature")
                        .suggestion("Are you sure that this file contains encoded data?"));
                }
            }
            //Extract legth
            let mut len_vec: [u8; 4] = [0; 4];
            let mut len_vec_count: usize = 0;
            for line in &mut lines {
                let line = line?;
                if REGEX_PATTERN.is_match(&line) && len_vec_count < 4 {
                    len_vec[len_vec_count] = decode_line(&line)?;
                    len_vec_count += 1;
                }
                if len_vec_count >= 4 {
                    break;
                }
            }
            if len_vec_count != 4 {
                return Err(eyre!("Data len u32 too short").suggestion("The file may be corrupted"));
            }

            let mut data_len = u32::from_le_bytes(len_vec);
            let mut data_vec: Vec<u8> = Vec::with_capacity(data_len as usize);
            //Decode
            for line in &mut lines {
                let line = line?;
                if REGEX_PATTERN.is_match(&line) && data_len > 0 {
                    data_vec.push(decode_line(&line)?);
                    data_len -= 1;
                }
            }
            let mut lzma2_reader =
                Lzma2Reader::new(data_vec.as_slice(), LzmaOptions::DICT_SIZE_DEFAULT, None);
            io::copy(&mut lzma2_reader, &mut data_file)?;
            data_file.flush()?;
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
