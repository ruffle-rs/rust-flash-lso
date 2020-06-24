## [flash-lso](https://crates.io/crates/flash-lso)

[![GitHub license](https://img.shields.io/github/license/CUB3D/rust-sol)](https://github.com/CUB3D/rust-flash-lso/blob/master/LICENSE)
[![GitHub issues](https://img.shields.io/github/issues/CUB3D/rust-sol)](https://github.com/CUB3D/rust-flash-lso/issues)

A parser for Adobe LSO (.sol), AMF0 and AFM3 in 100% safe rust

The primary goal of this crate is to be as safe as possible against malformed and invalid input and to fail cleanly when this is identified.

In future this crate also intends to support encoding of data to LSO/AMF0/AMF3


## Example
```rust
use std::fs::File;
use std::io::Read;
use flash_lso::LSODeserializer;
fn main() {
    let mut x = File::open(path).expect("Couldn't open file");
    let mut data = Vec::new();
    let _ = x.read_to_end(&mut data).expect("Unable to read file");
    let d = LSODeserializer::default().parse_full(&data).expect("Failed to parse lso file");
    println!("{:#?}", d);
}
``` 

## Development / Testing
To aid with development, there is a sub-project: reader, which can parse either a single file or all files in a directory and will report on which files succeeded and failed to parse

In future this will be used to verify and compare the output with the official implementation

## Fuzzing
This project makes use of cargo-fuzz to ensure correct handling of invalid data
```
cargo fuzz run --release fuzz_amf3_body
```

## License
This project is licensed under MIT
