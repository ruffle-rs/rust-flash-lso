#![no_main]
use libfuzzer_sys::fuzz_target;

use amf::amf3;

fuzz_target!(|data: &[u8]| {
    let _ = amf3::AMF3Decoder::default().parse_body(data);
});
