#![no_main]
use libfuzzer_sys::fuzz_target;

use flash_lso::amf0::decoder;

fuzz_target!(|data: &[u8]| {
    decoder::parse_body(data);
});
