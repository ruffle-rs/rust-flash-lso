use crate::blob_bindgen::Blob;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub type URL;

    #[wasm_bindgen(static_method_of=URL)]
    pub fn createObjectURL(blob: &Blob) -> String;
}
