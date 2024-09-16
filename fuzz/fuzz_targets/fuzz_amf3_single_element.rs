//! This specifically tests the `parse_single_element` function used for
//! `ByteArray.readObject` in Ruffle among other places
#![no_main]
use libfuzzer_sys::fuzz_target;

use flash_lso::amf3;

fuzz_target!(|data: &[u8]| {
    let _ = amf3::read::AMF3Decoder::default().parse_single_element(data);
});
