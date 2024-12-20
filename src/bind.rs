use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(module = "/js/bind.js")]
extern "C" {
    pub fn get_url_fragment() -> String;
}