#![no_main]
use libfuzzer_sys::fuzz_target;

use flash_lso::read::Reader;

fuzz_target!(|data: &[u8]| {
    Reader::default().parse_header(data);
});
