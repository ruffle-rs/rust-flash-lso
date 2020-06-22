#![no_main]
use libfuzzer_sys::fuzz_target;

use amf::amf3;

fuzz_target!(|data: &[u8]| {
    amf3::read_int_signed(data);
});
