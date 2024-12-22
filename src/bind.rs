use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(module = "/js/bind.js")]
extern "C" {
    pub fn get_url_fragment() -> String;

    pub fn log_string(s: &str);
}

macro_rules! console_log {
    // Note that this is using the `log` function imported above during
    // `bare_bones`
    ($($t:tt)*) => ($crate::bind::log_string(&format_args!($($t)*).to_string()))
}
