use js_sys::Array;
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub type Blob;

    #[wasm_bindgen(constructor)]
    pub fn new(text: Array, options: JsValue) -> Blob;
}
