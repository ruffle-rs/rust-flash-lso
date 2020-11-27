use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub type Uint8Array;

    #[wasm_bindgen(constructor)]
    pub fn new(length: u32) -> Uint8Array;

    #[wasm_bindgen(method, structural, indexing_setter)]
    pub fn set(this: &Uint8Array, prop: u32, val: u32);
}
