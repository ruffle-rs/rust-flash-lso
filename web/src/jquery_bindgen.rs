use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    pub type JQueryValue;

    #[wasm_bindgen(js_name = "$")]
    pub fn jquery(name: &str) -> JQueryValue;

    #[wasm_bindgen(method)]
    pub fn jstree(this: &JQueryValue);
}
