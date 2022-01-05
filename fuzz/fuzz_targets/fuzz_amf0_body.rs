#![no_main]
use libfuzzer_sys::fuzz_target;

use flash_lso::amf0::read::AMF0Decoder;

fuzz_target!(|data: &[u8]| {
    AMF0Decoder::default().parse_body(data);
});
