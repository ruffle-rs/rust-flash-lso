use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use clap::*;
use flash_lso::flex::decode;
use flash_lso::types::Sol;
use flash_lso::LSODeserializer;

fn main() {
    env_logger::init();

    let matched = App::new("LSO -> json converter")
        .version("1.0")
        .author("CUB3D <callumthom11@gmail.com>")
        .arg(Arg::with_name("INPUT").help("").required(true))
        .get_matches();

    let file_name = matched.value_of("INPUT").unwrap();

    let file = Path::new(file_name);
    if let Some(sol) = read_file(file.into()) {
        let json = serde_json::to_string(&sol).expect("Unable to encode lso as json");
        println!("{}", json);
    } else {
        eprintln!("Couldn't read lso file, maybe open a issue on github at https://github.com/CUB3D/rust-flash-lso");
    }
}

fn read_file(path: PathBuf) -> Option<Sol> {
    let mut x = File::open(path).unwrap();
    let mut data = Vec::new();
    let _ = x.read_to_end(&mut data).expect("Unable to read file");
    let mut d = LSODeserializer::default();
    decode::register_decoders(&mut d.amf3_decoder);

    let d = d.parse(&data);
    d.map(|s| s.1).ok()
}
