# ADR-001: MathScribe Architecture — NL→Formula Pipeline

## Status
Accepted

## Context

Scriptorium needs inline formula support. Users write mathematical expressions in natural language (Portuguese, English, Spanish) and expect a real-time preview of the rendered formula. The conversion must be deterministic (no LLM dependency), fast (<16ms for 60fps preview), and work both on desktop (native) and web (WASM).

## Decision

### 1. Three-Stage Pipeline

```
NL Text → [Tokenizer] → Tokens → [Parser] → MathAST → [Renderer] → Output
```

**Tokenizer** — Language-aware lexical analysis:
- Input: raw NL string + optional language hint
- Language detection via keyword matching (e.g., "quadrado" → PT, "squared" → EN, "cuadrado" → ES)
- Output: `Vec<Token>` where Token = Number | Variable | Keyword | Operator | Whitespace | Unknown
- Must handle mixed content: "a integral de x^2 + 1" (NL + symbolic math)

**Parser** — Grammar-driven recursive descent:
- Consumes Token stream using Grammar trait for the detected language
- Produces `MathAST` (tree of `MathExpr` nodes)
- No backtracking — each token is consumed once, O(n) parsing
- Fallback: unrecognized patterns become `Variable` nodes (never fail)

**Renderer** — AST → output format:
- Trait `Renderer` with `fn render(&self, expr: &MathExpr) -> String`
- Three implementations:
  - `UnicodeRenderer` — fast, for live preview (Σ, √, ², superscripts)
  - `LatexRenderer` — precise, for export (\frac{}{}, \sqrt{}, \int)
  - `MathMLRenderer` — for web accessibility (<math>, <msup>, <mfrac>)

### 2. Grammar Trait (Multi-Language)

```rust
trait Grammar {
    fn language(&self) -> Language;
    fn keywords(&self) -> &[KeywordPattern];
    fn parse_expression(&self, tokens: &[Token], pos: usize) -> ParseResult;
}
```

Each language implements `Grammar`:
- `PortugueseGrammar` — "ao quadrado", "raiz de", "integral de ... até", "soma de"
- `EnglishGrammar` — "squared", "square root of", "integral of ... to", "sum from"
- `SpanishGrammar` — "al cuadrado", "raíz de", "integral de ... hasta", "suma de"

**Language Detection** — automatic from keywords:
- Scan first N tokens for language-specific keywords
- If ambiguous (e.g., "integral" is the same in all three), use last confirmed detection or default
- Can be overridden by caller: `parse_with_language(input, Language::Portuguese)`

### 3. MathAST

The AST is a recursive enum `MathExpr` covering:
- Literals: `Number(f64)`, `Variable(String)`, `Constant(Pi, E, Infinity)`
- Binary ops: `Add`, `Sub`, `Mul`, `Div`, `Power`
- Functions: `Function(MathFunc, expr)` — sin, cos, tan, log, ln, exp
- Roots: `Root(expr, degree)` — √x (degree=2), ∛x (degree=3)
- Calculus: `Integral`, `Derivative`, `Limit`, `Sum`, `Product` — each with bounds
- Relations: `Equals`, `LessThan`, `GreaterThan`, `LessOrEqual`, `GreaterOrEqual`, `ElementOf`
- Grouping: `Parenthesized`, `Fraction`

The AST is:
- Serializable (serde) for debugging and testing
- Traversable (visitor pattern optional, direct match for now)
- WASM-compatible (no Rc, no RefCell — owned Box<> pointers)

### 4. WASM Strategy

```
mathscribe-core    → Pure Rust, no system deps, wasm32-unknown-unknown
mathscribe-wasm    → wasm-bindgen layer, exports JS functions
```

Exported WASM functions:
- `parse(input: &str, language: Option<&str>) -> JsValue` — returns { ast: MathExpr, latex: String, unicode: String }
- `render_latex(input: &str) -> String`
- `render_unicode(input: &str) -> String`
- `detect_language(input: &str) -> String`

Size budget: < 200KB gzipped WASM binary.

### 5. Scriptorium Integration

MathScribe implements Scriptorium's `MathParser` trait (defined in Scriptorium US-039):

```rust
// Scriptorium defines:
trait MathParser {
    fn parse(&self, input: &str) -> MathResult;
}

struct MathResult {
    source_nl: String,
    rendered_unicode: String,
    rendered_latex: String,
}

// MathScribe provides:
struct MathScribeParser {
    grammars: Vec<Box<dyn Grammar>>,
    unicode_renderer: UnicodeRenderer,
    latex_renderer: LatexRenderer,
}

impl MathParser for MathScribeParser {
    fn parse(&self, input: &str) -> MathResult {
        let tokens = self.tokenize(input);
        let lang = self.detect_language(&tokens);
        let grammar = self.grammar_for(lang);
        let ast = grammar.parse_expression(&tokens, 0);
        MathResult {
            source_nl: input.to_string(),
            rendered_unicode: self.unicode_renderer.render(&ast),
            rendered_latex: self.latex_renderer.render(&ast),
        }
    }
}
```

On desktop: Scriptorium adds `mathscribe-core` as a Cargo dependency.
On web: Scriptorium loads `mathscribe-wasm` as a WASM module.

### 6. Error Handling

MathScribe never panics. Errors are:
- `UnrecognizedPattern` — fallback to Variable node, log warning
- `AmbiguousExpression` — return best guess + list of alternatives
- `MalformedInput` — return partial AST up to the error point

The caller (Scriptorium) decides how to display errors in the UI.

## Consequences

- **Positive:** Deterministic, fast, no network dependency. Works offline.
- **Positive:** Multi-language from day one. Adding a language = implementing Grammar.
- **Positive:** Three output formats cover all use cases (preview, export, accessibility).
- **Negative:** Rule-based parsing won't handle every possible NL phrasing. Mitigation: fallback to Variable + future LLM-assisted disambiguation via slm-engine jobs.
- **Negative:** Grammar maintenance per language. Mitigation: comprehensive test suites per language.

## References

- Scriptorium `docs/Phase-3-User-Stories.md` US-039 (MathParser trait)
- Scriptorium `CLAUDE.md` (Action Bar MathScribe mode)
- `docs/Phase-4-User-Stories.md` (this project's US)
