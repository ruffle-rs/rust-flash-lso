## [flash-lso](https://crates.io/crates/flash-lso)

[![GitHub license](https://img.shields.io/github/license/CUB3D/rust-sol)](https://github.com/CUB3D/rust-flash-lso/blob/master/LICENSE)
[![GitHub issues](https://img.shields.io/github/issues/CUB3D/rust-sol)](https://github.com/CUB3D/rust-flash-lso/issues)

A parser/encoder for Adobe Local Shared Object (LSO) file format (.sol), AMF0 and AFM3 in 100% safe rust.
#### Features:
- Parsing and encoding fully supported
- Heavily tested and fuzzed
- Circular references fully supported
- Support for externalizable types (flash.utils.IExternalizable)
- Support for Adobe flex types

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
This project is licensed under MIT.

Icons used in the web editor (web/static/icon) are sourced from https://feathericons.com under MIT
