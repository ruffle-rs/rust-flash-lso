#![no_main]
use libfuzzer_sys::fuzz_target;

use flash_lso::amf3;

fuzz_target!(|data: &[u8]| {
    let _ = amf3::read::AMF3Decoder::default().parse_body(data);
});
