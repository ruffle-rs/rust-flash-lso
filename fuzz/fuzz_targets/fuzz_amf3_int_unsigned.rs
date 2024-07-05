#![no_main]
use libfuzzer_sys::fuzz_target;

use flash_lso::amf3::read::fuzz_read_int;

fuzz_target!(|data: &[u8]| {
    let _ = fuzz_read_int(data);
});
