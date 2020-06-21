use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use amf::Sol;
use clap::*;

fn main() {
    env_logger::init();

    let matched = App::new("SOL file reader")
        .version("1.0")
        .author("CUB3D <callumthom11@gmail.com>")
        .arg(Arg::with_name("INPUT").help("").required(true))
        .get_matches();

    let file_name = matched.value_of("INPUT").unwrap();

    let file = Path::new(file_name);
    if file.is_dir() {
        let children = file.read_dir().unwrap();
        for child in children {
            if let Ok(c) = child {
                let worked = read_file(c.path());
                if worked.is_some() {
                    println!("{} = Worked", c.file_name().to_string_lossy().to_string())
                } else {
                    println!("{} = Failed", c.file_name().to_string_lossy().to_string())
                }
            }
        }
    } else {
        read_file(file.into());
    }
}

fn read_file(path: PathBuf) -> Option<Sol> {
    let mut x = File::open(path).unwrap();
    let mut data = Vec::new();
    let _ = x.read_to_end(&mut data).expect("Unable to read file");
    let d = amf::parse_full(&data);
    println!("{:#?}", d);
    d.map(|s| s.1).ok()
}
