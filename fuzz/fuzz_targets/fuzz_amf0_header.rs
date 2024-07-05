#![no_main]
use libfuzzer_sys::fuzz_target;

use flash_lso::read::Reader;

fuzz_target!(|data: &[u8]| {
    let _ = Reader::default().parse_header(data);
});
