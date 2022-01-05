//! This tool allows converting a given Flash Local Shared Object file (Lso) to a JSON document for
//! easy previewing, as well as the creation of test cases for the flash-lso library

#![deny(missing_docs, clippy::missing_docs_in_private_items)]

use clap::{App, Arg};
use flash_lso::extra::*;
use flash_lso::read::Reader;
use flash_lso::types::Lso;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let matched = App::new("Lso -> json converter")
        .version("1.0")
        .author("CUB3D <callumthom11@gmail.com>")
        .arg(Arg::new("INPUT").help("").required(true))
        .get_matches();

    let file_name = matched.value_of("INPUT").unwrap();

    let data = std::fs::read(PathBuf::from(file_name))?;

    match parse_file(&data) {
        Ok(lso) => {
            let json = serde_json::to_string(&lso).expect("Unable to encode lso as json");
            println!("{}", json);
        }
        Err(e) => {
            eprintln!("Couldn't read lso file, maybe open a issue on github at https://github.com/CUB3D/rust-flash-lso");
            eprintln!("Error = {:?}", e);
        }
    }

    Ok(())
}

/// Parse a given slice into an Lso
fn parse_file(data: &[u8]) -> Result<Lso, Box<dyn std::error::Error + '_>> {
    let mut d = Reader::default();
    flex::read::register_decoders(&mut d.amf3_decoder);
    let lso = d.parse(data)?;
    Ok(lso)
}
