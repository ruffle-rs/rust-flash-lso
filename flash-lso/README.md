## [flash-lso](https://crates.io/crates/flash-lso)

[![GitHub license](https://img.shields.io/github/license/CUB3D/rust-sol)](https://github.com/CUB3D/rust-flash-lso/blob/master/LICENSE)
[![GitHub issues](https://img.shields.io/github/issues/CUB3D/rust-sol)](https://github.com/CUB3D/rust-flash-lso/issues)

A parser and serializer for Adobe Local Shared Object (LSO) file format (.sol), AMF0 and AFM3 in 100% safe rust

The primary goal of this crate is to be as safe as possible against malformed and invalid input and to fail cleanly when this is identified.

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

## Web
The ```web``` directory contains an example web viewer for LSO files using yew. To run, first build into WASM like so
```shell script
cd web
wasm-pack build --out-name wasm --out-dir ./static --target web --release
```
Then serve the static directory like so
```shell script
# If needed install miniserve with `cargo install miniserve`
miniserve ./static --index index.html
```

## Development / Testing
This project has a collection of integration tests to verify that it is able to serialize and then deserialize LSO files to produce output that is identical to it's input
Also available is a lso-to-json project which allows dumping an LSO file to json for debugging and testing.

## Serde
To enable serde support, enable the serde feature like so
```toml
flash-lso = { version = "0.2.0", features = ["serde"] }
```

## Fuzzing
This project makes use of cargo-fuzz to ensure correct handling of invalid data
```
cargo fuzz run --release fuzz_amf3_body
```

## License
This project is licensed under MIT
