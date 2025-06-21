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
use flash_lso::read::Reader;
fn main() {
    let mut x = File::open(path).expect("Couldn't open file");
    let mut data = Vec::new();
    let _ = x.read_to_end(&mut data).expect("Unable to read file");
    let d = Reader::default().parse_full(&data).expect("Failed to parse lso file");
    println!("{:#?}", d);
}
```

## Fuzzing
This project makes use of cargo-fuzz to ensure correct handling of invalid data
```
cargo +nightly fuzz run --release fuzz_amf3_body
```

## Web
building:
```
cd web/
wasm-pack build --out-name web --out-dir ./static --target web --release
miniserve ./static --index index.html
```

## License
This project is licensed under MIT.

Icons used in the web editor (web/static/icon) are sourced from https://feathericons.com under MIT

Some test cases are covered under their own License, see [README.md](flash-lso/tests/README.md) for details 
