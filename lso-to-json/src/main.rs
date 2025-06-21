//! This tool allows converting a given Flash Local Shared Object file (Lso) to a JSON document for
//! easy previewing, as well as the creation of test cases for the flash-lso library

#![deny(missing_docs, clippy::missing_docs_in_private_items)]

use clap::{Arg, Command};
use flash_lso::amf3::read::AMF3Decoder;
use flash_lso::extra::*;
use flash_lso::read::Reader;
use flash_lso::types::Lso;
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let matched = Command::new("Lso -> json converter")
        .version("1.0")
        .author("CUB3D <callumthom11@gmail.com>")
        .subcommand(Command::new("file").arg(Arg::new("INPUT").help("").required(true)))
        .subcommand(Command::new("object-amf3").arg(Arg::new("INPUT").help("").required(true)))
        .subcommand(Command::new("regen").arg(Arg::new("INPUT").help("").required(true)))
        .subcommand_required(true)
        .get_matches();

    let (cmd, args) = matched.subcommand().unwrap();

    let file_name: &String = args.get_one("INPUT").unwrap();

    match cmd {
        "file" => {
            let data = std::fs::read(PathBuf::from(file_name))?;
            match parse_file(&data) {
                Ok(lso) => {
                    let json = serde_json::to_string(&lso).expect("Unable to encode lso as json");
                    println!("{}", json);
                }
                Err(e) => {
                    eprintln!(
                        "Couldn't read lso file, maybe open a issue on github at https://github.com/CUB3D/rust-flash-lso"
                    );
                    eprintln!("Error = {:?}", e);
                }
            };
        }
        "regen" => {
            for f in std::fs::read_dir(file_name)? {
                let f = f?;

                if !f.file_type()?.is_file() {
                    continue;
                }
                let name = f.file_name().to_string_lossy().to_string();

                if name.ends_with(".sol") {
                    let data = std::fs::read(f.path())?;

                    let out = f
                        .path()
                        .parent()
                        .unwrap()
                        .join(f.file_name().to_string_lossy().replace(".sol", ".json"));

                    match parse_file(&data) {
                        Ok(lso) => {
                            let json =
                                serde_json::to_string(&lso).expect("Unable to encode lso as json");
                            std::fs::write(out, format!("{}\n", json))
                                .expect("Unable to write file");
                        }
                        Err(e) => {
                            eprintln!(
                                "Couldn't read lso file, maybe open a issue on github at https://github.com/CUB3D/rust-flash-lso"
                            );
                            eprintln!("Error = {:?}", e);
                        }
                    };
                } else if name.ends_with(".amf") {
                    let data = std::fs::read(f.path())?;

                    let out = f
                        .path()
                        .parent()
                        .unwrap()
                        .join(f.file_name().to_string_lossy().replace(".amf", ".json"));

                    match AMF3Decoder::default().parse_single_element(&data) {
                        Ok((_, amf)) => {
                            let json =
                                serde_json::to_string(&amf).expect("Unable to encode amf as json");
                            std::fs::write(out, json).expect("Unable to write file");
                        }
                        Err(e) => {
                            eprintln!(
                                "Couldn't read amf file, maybe open a issue on github at https://github.com/CUB3D/rust-flash-lso"
                            );
                            eprintln!("Error = {:?}", e);
                        }
                    };
                }
            }
        }
        "object-amf3" => {
            let data = std::fs::read(PathBuf::from(file_name))?;
            let (_, obj) = AMF3Decoder::default()
                .parse_single_element(&data)
                .expect("Failed to parse object");
            let json = serde_json::to_string(&obj).expect("Unable to encode lso as json");

            println!("{}", json);
        }
        _ => {
            println!("Unknown command");
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
