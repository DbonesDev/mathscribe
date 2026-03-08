# MathScribe

A natural language to mathematical formula conversion engine, written in Rust.

MathScribe accepts input in **Portuguese**, **English**, and **Spanish**, and produces output in **LaTeX**, **Unicode**, and **MathML**.

> The name combines "Math" with "Scribe" — the engine that transcribes the student's mathematical intent into formal notation.

## Examples

| Input (Natural Language) | Output (LaTeX) | Output (Unicode) |
|---|---|---|
| `x ao quadrado` (PT) | `x^{2}` | x² |
| `square root of x` (EN) | `\sqrt{x}` | √x |
| `a sobre b` (PT/ES) | `\frac{a}{b}` | a/b |
| `integral of x` (EN) | `\int x` | ∫x |
| `x al cuadrado` (ES) | `x^{2}` | x² |

## Architecture

```
NL Text → [Tokenizer] → Tokens → [Parser] → MathAST → [Renderer] → LaTeX / Unicode / MathML
```

1. **Tokenizer** — Receives NL text, detects language, produces a token stream
2. **Parser** — Consumes tokens using language-specific grammar rules, builds a MathAST (recursive descent)
3. **Renderer** — Traverses MathAST, produces output in the desired format

## Project Structure

```
mathscribe/
├── crates/
│   ├── core/       # Pure Rust engine: tokenizer, parser, AST, renderer, grammars
│   └── wasm/       # WASM bindings via wasm-bindgen
├── docs/           # ADRs and user stories
├── Cargo.toml      # Workspace
└── CLAUDE.md       # AI development context
```

## Building

```bash
# Build the workspace
cargo build --workspace

# Run all tests
cargo test --workspace

# Lint
cargo clippy --workspace -- -D warnings

# Format
cargo fmt --all

# Verify WASM compilation
cargo build -p mathscribe-core --target wasm32-unknown-unknown

# Build WASM package for browser
cd crates/wasm && wasm-pack build --target web
```

## Part of the Arcasidian Ecosystem

MathScribe is a standalone project within the [Arcasidian](https://github.com/DbonesDev) ecosystem. It is consumed by **Scriptorium** (essay editor) as a dependency and can be embedded in web applications via its WASM module.

## License

MIT
