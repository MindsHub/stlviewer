use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(module = "/js/bind.js")]
extern "C" {
    pub fn get_url_fragment() -> String;

    pub fn console_log(s: &str);
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => ($crate::bind::console_log(&format_args!($($t)*).to_string()))
}
