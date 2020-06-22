#![no_main]
use libfuzzer_sys::fuzz_target;

use amf::amf0;

fuzz_target!(|data: &[u8]| {
    let _ = amf0::parse_signature(data);
});
