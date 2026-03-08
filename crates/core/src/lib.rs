//! MathScribe Core — NL→Formula Engine
//!
//! Converts natural language mathematical expressions in Portuguese, English,
//! and Spanish into structured formula representations (LaTeX, Unicode, MathML).
//!
//! # Architecture
//!
//! ```text
//! Input (NL text) → Parser → MathAST → Renderer → Output (LaTeX/Unicode/MathML)
//! ```
//!
//! The parser is language-aware and uses pattern matching + grammar rules
//! to build a `MathAST`. The renderer converts the AST into the desired format.
//!
//! # WASM Compatibility
//!
//! This crate is `no_std`-friendly and compiles to `wasm32-unknown-unknown`.
//! No filesystem, network, or system dependencies.

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert_eq!(version(), "0.1.0");
    }
}
