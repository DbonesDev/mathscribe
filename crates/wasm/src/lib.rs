//! MathScribe WASM — Browser bindings for the NL→Formula engine
//!
//! Provides JavaScript-callable functions via wasm-bindgen.
//! Used by Scriptorium (web) and Arca School platform.

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn mathscribe_version() -> String {
    mathscribe_core::version().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_version() {
        assert_eq!(mathscribe_version(), "0.1.0");
    }
}
