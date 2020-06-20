use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn main() {
    env_logger::init();

    let mut x = File::open(Path::new("../ballSave.sol")).unwrap();
    let mut data = Vec::new();
    let _ = x.read_to_end(&mut data).unwrap();
    let (i, sol) = amf::parse_full(&data).unwrap();
    println!("{:#?}", sol);

    let mut x = File::open(Path::new("../NiceTestFile.sol")).unwrap();
    let mut data = Vec::new();
    let _ = x.read_to_end(&mut data).unwrap();
    let sol = amf::parse_full(&data);
    println!("{:#?}", sol);
}
