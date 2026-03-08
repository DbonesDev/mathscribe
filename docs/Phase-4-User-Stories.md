# Q2-Q3 2026: MathScribe — NL→Formula Engine — User Stories

> **Prerequisite:** Scriptorium Q2 2026 Sprint 3 concluído (document model, markdown roundtrip). MathParser trait definido (US-039).
> **Goal:** Construir um motor determinístico que converte linguagem natural (PT/EN/ES) em fórmulas matemáticas (LaTeX, Unicode, MathML).
> **ADR:** Follow `ADR-001-mathscribe-architecture.md`.
> **Workflow:** Follow `DDI-INSTRUCTIONS.md` for all implementation steps.
> **Repos:** `mathscribe` (standalone crate) — separate project from Scriptorium.
> **Release:** mathscribe v0.1 at the end of this phase.
> **Design:** No UI — this is a pure library. UI lives in Scriptorium.

---

## Sprint 10 (Foundation): MathAST + Tokenizer

### US-MS-001: Create `mathscribe` repo and architecture ADR

- **Como** developer
- **Quero** um repo standalone para o motor NL→fórmula com ADR definindo a arquitectura
- **Para que** MathScribe exista como produto independente reutilizável

**DDI Intent:**
```
Quero criar o projecto MathScribe como repo Rust standalone.
Ele deve:
- Ser um Cargo workspace com crates: mathscribe-core, mathscribe-wasm
- Ter ADR-001-mathscribe-architecture.md definindo:
  - Pipeline de 3 estágios: Tokenizer → Parser → Renderer
  - Grammar trait para multi-language
  - Renderer trait para multi-format
  - WASM strategy
- Ter CI com fmt + clippy + tests + WASM build check
- Ter CLAUDE.md com contexto do projecto
- Ter DDI-INSTRUCTIONS.md
- NÃO depender do Scriptorium (standalone)
```

**Test Criteria:**
- [ ] `cargo build --workspace` compila ambos crates
- [ ] `mathscribe-core` compila com `--target wasm32-unknown-unknown`
- [ ] ADR-001 documenta: pipeline, Grammar trait, Renderer trait, WASM strategy
- [ ] CI configurado: fmt + clippy + tests
- [ ] CLAUDE.md com contexto completo
- [ ] `cargo clippy --workspace -- -D warnings` zero warnings
- [ ] `cargo test --workspace` passa

**Estimativa:** 1 sessão DDI (~30min)

---

### US-MS-002: MathAST — Expression Tree

- **Como** developer
- **Quero** uma árvore de expressões matemáticas com todos os tipos necessários
- **Para que** o parser tenha onde guardar o resultado e o renderer tenha o que percorrer

**DDI Intent:**
```
Quero criar o MathAST no mathscribe-core.
Ele deve:
- Definir enum MathExpr com variantes:
  - Literais: Number(f64), Variable(String), Constant(MathConstant)
  - Binários: Add, Sub, Mul, Div, Power (cada com Box<MathExpr>, Box<MathExpr>)
  - Funções: Function(MathFunc, Box<MathExpr>) — sin, cos, tan, log, ln, exp
  - Raízes: Root(Box<MathExpr>, Option<Box<MathExpr>>) — √x, ∛x
  - Cálculo: Integral, Derivative, Limit, Sum, Product — com bounds
  - Relações: Equals, LessThan, GreaterThan, LessOrEqual, GreaterOrEqual, ElementOf
  - Agrupamento: Parenthesized, Fraction
- Definir enum MathConstant: Pi, E, Infinity
- Definir enum MathFunc: Sin, Cos, Tan, Log, Ln, Exp, Abs
- Todas as structs devem ser serde Serialize + Deserialize
- Todas devem ser Clone + Debug + PartialEq
- Nenhuma usa Rc ou RefCell (WASM-compatible, owned pointers)
```

**Test Criteria:**
- [ ] Teste: `MathExpr::Number(42.0)` serializa para JSON e deserializa correctamente
- [ ] Teste: `MathExpr::Add(Number(1), Number(2))` — round-trip serde
- [ ] Teste: `MathExpr::Power(Variable("x"), Number(2))` — representa x²
- [ ] Teste: `MathExpr::Integral { body, var, lower, upper }` — com e sem bounds
- [ ] Teste: `MathExpr::Sum { body, var, lower, upper }` — round-trip serde
- [ ] Teste: `MathExpr::Function(Sin, Variable("x"))` — sin(x)
- [ ] Teste: `MathExpr::Root(Variable("x"), None)` — √x (default square root)
- [ ] Teste: `MathExpr::Root(Variable("x"), Some(Number(3)))` — ∛x
- [ ] Teste: nested expressions — `(x² + 1) / (x - 3)` como AST
- [ ] Teste: `MathConstant::Pi` serializa como `"pi"`
- [ ] Compilável para WASM
- [ ] `cargo clippy` zero warnings

**Estimativa:** 1 sessão DDI (~45min)

---

### US-MS-003: Tokenizer — NL Text → Token Stream

- **Como** developer
- **Quero** um tokenizer que decomponha texto NL + math em tokens tipados
- **Para que** o parser receba uma stream limpa e estruturada

**DDI Intent:**
```
Quero um Tokenizer no mathscribe-core.
Ele deve:
- Definir enum Token: Number(f64), Variable(String), Keyword(String), Operator(Op),
  LeftParen, RightParen, Whitespace, Unknown(char)
- Definir enum Op: Plus, Minus, Times, Divide, Caret, Equals, LessThan, GreaterThan
- Método tokenize(input: &str) -> Vec<Token>:
  - Números: "42", "3.14", "-7" → Number
  - Variáveis: "x", "y", "theta" → Variable
  - Operadores: "+", "-", "*", "/", "^", "=", "<", ">" → Operator
  - Parênteses: "(", ")" → LeftParen, RightParen
  - Keywords (multi-word): "ao quadrado", "square root", "al cuadrado" → Keyword
  - Tudo o resto: Unknown
- Método detect_language(tokens: &[Token]) -> Language:
  - Scan keywords nos tokens
  - "quadrado", "raiz", "sobre", "até", "soma" → Portuguese
  - "squared", "root", "over", "from", "sum" → English
  - "cuadrado", "raíz", "hasta", "suma" → Spanish
  - Default: English se ambíguo
- Keywords são case-insensitive
- Suporta caracteres Unicode: π, θ, α, β, γ, etc.
```

**Test Criteria:**
- [ ] Teste: `"42"` → `[Number(42.0)]`
- [ ] Teste: `"3.14"` → `[Number(3.14)]`
- [ ] Teste: `"x + y"` → `[Variable("x"), Operator(Plus), Variable("y")]`
- [ ] Teste: `"x^2"` → `[Variable("x"), Operator(Caret), Number(2)]`
- [ ] Teste: `"(a + b)"` → `[LeftParen, Variable("a"), Operator(Plus), Variable("b"), RightParen]`
- [ ] Teste: `"x ao quadrado"` → `[Variable("x"), Keyword("ao quadrado")]`
- [ ] Teste: `"square root of x"` → `[Keyword("square root"), Keyword("of"), Variable("x")]`
- [ ] Teste: `"x al cuadrado"` → `[Variable("x"), Keyword("al cuadrado")]`
- [ ] Teste: detect_language com "ao quadrado" → Portuguese
- [ ] Teste: detect_language com "squared" → English
- [ ] Teste: detect_language com "al cuadrado" → Spanish
- [ ] Teste: detect_language sem keywords → English (default)
- [ ] Teste: `"π"` → `[Variable("π")]`
- [ ] Teste: `"θ + 1"` → `[Variable("θ"), Operator(Plus), Number(1)]`
- [ ] Teste: case insensitive: `"Ao Quadrado"` = `"ao quadrado"`
- [ ] Compilável para WASM

**Estimativa:** 2 sessões DDI (~2h)

---

## Sprint 11 (Grammars): Language-Specific Parsers

### US-MS-004: Portuguese Grammar

- **Como** aluno português
- **Quero** escrever fórmulas em português natural e vê-las convertidas
- **Para que** a barreira linguística não impeça a expressão matemática

**DDI Intent:**
```
Quero uma PortugueseGrammar que implemente o trait Grammar.
Ela deve reconhecer os seguintes padrões:

Aritmética:
- "x mais y" → Add(x, y)
- "x menos y" → Sub(x, y)
- "x vezes y" → Mul(x, y)
- "x dividido por y" → Div(x, y)
- "x ao quadrado" → Power(x, 2)
- "x ao cubo" → Power(x, 3)
- "x à potência de n" → Power(x, n)
- "a sobre b" → Fraction(a, b)

Raízes:
- "raiz quadrada de x" → Root(x, 2)
- "raiz cúbica de x" → Root(x, 3)

Funções:
- "seno de x" → Function(Sin, x)
- "cosseno de x" → Function(Cos, x)
- "tangente de x" → Function(Tan, x)
- "logaritmo de x" → Function(Log, x)
- "logaritmo natural de x" → Function(Ln, x)

Cálculo:
- "integral de f" → Integral { body: f, var: implicit }
- "integral de a até b de f dx" → Integral { body: f, var: "x", lower: a, upper: b }
- "soma de i=1 até n de f" → Sum { body: f, var: "i", lower: 1, upper: n }
- "produto de i=1 até n de f" → Product { body: f, var: "i", lower: 1, upper: n }
- "limite de f quando x tende a c" → Limit { body: f, var: "x", approaching: c }
- "derivada de f em relação a x" → Derivative { body: f, var: "x", order: 1 }

Relações:
- "x igual a y" → Equals(x, y)
- "x maior que y" → GreaterThan(x, y)
- "x menor que y" → LessThan(x, y)
- "x maior ou igual a y" → GreaterOrEqual(x, y)
- "x menor ou igual a y" → LessOrEqual(x, y)
- "x pertence a R" → ElementOf(x, Variable("ℝ"))

Constantes:
- "pi" → Constant(Pi)
- "infinito" → Constant(Infinity)
```

**Test Criteria:**
- [ ] Teste: `"x ao quadrado"` → `Power(Variable("x"), Number(2))`
- [ ] Teste: `"x ao cubo"` → `Power(Variable("x"), Number(3))`
- [ ] Teste: `"raiz quadrada de x"` → `Root(Variable("x"), None)`
- [ ] Teste: `"a sobre b"` → `Fraction(Variable("a"), Variable("b"))`
- [ ] Teste: `"seno de x"` → `Function(Sin, Variable("x"))`
- [ ] Teste: `"integral de x ao quadrado"` → `Integral { body: Power(x, 2) }`
- [ ] Teste: `"soma de i=1 até n de i ao quadrado"` → `Sum { ... }`
- [ ] Teste: `"limite de 1 sobre x quando x tende a infinito"` → `Limit { ... }`
- [ ] Teste: `"derivada de x ao cubo em relação a x"` → `Derivative { body: Power(x, 3), var: "x" }`
- [ ] Teste: `"x maior ou igual a 0"` → `GreaterOrEqual(Variable("x"), Number(0))`
- [ ] Teste: `"x pertence a R"` → `ElementOf(Variable("x"), Variable("ℝ"))`
- [ ] Teste: expressão complexa: `"integral de 0 até pi de seno de x dx"` → AST correcto
- [ ] Teste: padrão não reconhecido → fallback para Variable
- [ ] Compilável para WASM

**Estimativa:** 3 sessões DDI (~3h)

---

### US-MS-005: English Grammar

- **Como** English-speaking student
- **Quero** to write formulas in natural English and see them converted
- **Para que** I can express mathematical ideas without knowing LaTeX syntax

**DDI Intent:**
```
Quero uma EnglishGrammar que implemente o trait Grammar.
Mesmos padrões que a PortugueseGrammar, mas em Inglês:

- "x squared" → Power(x, 2)
- "x cubed" → Power(x, 3)
- "x to the power of n" → Power(x, n)
- "a over b" → Fraction(a, b)
- "square root of x" → Root(x, 2)
- "cube root of x" → Root(x, 3)
- "sine of x" → Function(Sin, x)
- "cosine of x" → Function(Cos, x)
- "integral of f" → Integral { body: f }
- "integral from a to b of f dx" → Integral { bounds }
- "sum from i=1 to n of f" → Sum { bounds }
- "limit of f as x approaches c" → Limit
- "derivative of f with respect to x" → Derivative
- "x greater than or equal to y" → GreaterOrEqual
- "x belongs to R" → ElementOf
```

**Test Criteria:**
- [ ] Teste: `"x squared"` → `Power(Variable("x"), Number(2))`
- [ ] Teste: `"square root of x"` → `Root(Variable("x"), None)`
- [ ] Teste: `"a over b"` → `Fraction(Variable("a"), Variable("b"))`
- [ ] Teste: `"sine of x"` → `Function(Sin, Variable("x"))`
- [ ] Teste: `"integral from 0 to 1 of x squared dx"` → AST correcto
- [ ] Teste: `"sum from i=1 to n of i squared"` → Sum
- [ ] Teste: `"limit of 1 over x as x approaches infinity"` → Limit
- [ ] Teste: `"derivative of x cubed with respect to x"` → Derivative
- [ ] Teste: `"x greater than or equal to 0"` → GreaterOrEqual
- [ ] Teste: padrão não reconhecido → fallback
- [ ] Compilável para WASM

**Estimativa:** 2 sessões DDI (~2h) — faster than PT because patterns are established

---

### US-MS-006: Spanish Grammar

- **Como** estudiante hispanohablante
- **Quiero** escribir fórmulas en español natural y verlas convertidas
- **Para que** pueda expresar ideas matemáticas sin conocer la sintaxis LaTeX

**DDI Intent:**
```
Quero uma SpanishGrammar que implemente o trait Grammar.
Mesmos padrões, em Espanhol:

- "x al cuadrado" → Power(x, 2)
- "x al cubo" → Power(x, 3)
- "x a la potencia de n" → Power(x, n)
- "a sobre b" → Fraction(a, b)
- "raíz cuadrada de x" → Root(x, 2)
- "raíz cúbica de x" → Root(x, 3)
- "seno de x" → Function(Sin, x)
- "coseno de x" → Function(Cos, x)
- "integral de f" → Integral
- "integral de a hasta b de f dx" → Integral { bounds }
- "suma de i=1 hasta n de f" → Sum
- "límite de f cuando x tiende a c" → Limit
- "derivada de f respecto a x" → Derivative
- "x mayor o igual que y" → GreaterOrEqual
- "x pertenece a R" → ElementOf
```

**Test Criteria:**
- [ ] Teste: `"x al cuadrado"` → `Power(Variable("x"), Number(2))`
- [ ] Teste: `"raíz cuadrada de x"` → `Root(Variable("x"), None)`
- [ ] Teste: `"a sobre b"` → `Fraction(Variable("a"), Variable("b"))`
- [ ] Teste: `"seno de x"` → `Function(Sin, Variable("x"))`
- [ ] Teste: `"integral de 0 hasta 1 de x al cuadrado dx"` → AST correcto
- [ ] Teste: `"suma de i=1 hasta n de i al cuadrado"` → Sum
- [ ] Teste: `"límite de 1 sobre x cuando x tiende a infinito"` → Limit
- [ ] Teste: `"derivada de x al cubo respecto a x"` → Derivative
- [ ] Teste: `"x mayor o igual que 0"` → GreaterOrEqual
- [ ] Teste: padrão não reconhecido → fallback
- [ ] Compilável para WASM

**Estimativa:** 2 sessões DDI (~1h30) — patterns established from PT and EN

---

## Sprint 12 (Renderers): Output Formats

### US-MS-007: LaTeX + Unicode Renderers

- **Como** utilizador
- **Quero** ver a fórmula renderizada em Unicode (preview) e exportar em LaTeX
- **Para que** tenha feedback imediato e output compatível com KaTeX/MathJax

**DDI Intent:**
```
Quero dois renderers no mathscribe-core.

Trait Renderer:
  fn render(&self, expr: &MathExpr) -> String

UnicodeRenderer (para preview em tempo real):
- Number(42) → "42"
- Variable("x") → "x"
- Power(x, 2) → "x²"
- Power(x, 3) → "x³"
- Power(x, n) → "xⁿ" (superscript Unicode)
- Root(x, None) → "√x"
- Root(x, 3) → "∛x"
- Fraction(a, b) → "a/b" (simple) ou "ᵃ⁄ᵦ" (when possible)
- Function(Sin, x) → "sin(x)"
- Integral { } → "∫f dx" ou "∫ₐᵇ f dx"
- Sum { } → "Σᵢ₌₁ⁿ f"
- Product { } → "Πᵢ₌₁ⁿ f"
- Limit { } → "lim_{x→c} f"
- Derivative { } → "df/dx"
- GreaterOrEqual(x, y) → "x ≥ y"
- ElementOf(x, R) → "x ∈ ℝ"

LatexRenderer (para export):
- Power(x, 2) → "x^{2}"
- Root(x, None) → "\\sqrt{x}"
- Root(x, 3) → "\\sqrt[3]{x}"
- Fraction(a, b) → "\\frac{a}{b}"
- Function(Sin, x) → "\\sin(x)"
- Integral { lower, upper } → "\\int_{a}^{b} f \\, dx"
- Sum { } → "\\sum_{i=1}^{n} f"
- Limit { } → "\\lim_{x \\to c} f"
- Derivative { } → "\\frac{df}{dx}"
- GreaterOrEqual → "\\geq"
- ElementOf → "\\in"
- Constant(Pi) → "\\pi"
```

**Test Criteria:**
- [ ] Teste Unicode: `Power(Variable("x"), Number(2))` → `"x²"`
- [ ] Teste Unicode: `Root(Variable("x"), None)` → `"√x"`
- [ ] Teste Unicode: `Fraction(Variable("a"), Variable("b"))` → `"a/b"`
- [ ] Teste Unicode: `Sum { var: "i", lower: 1, upper: n }` → contém `"Σ"`
- [ ] Teste Unicode: `ElementOf(x, R)` → `"x ∈ ℝ"`
- [ ] Teste LaTeX: `Power(Variable("x"), Number(2))` → `"x^{2}"`
- [ ] Teste LaTeX: `Root(Variable("x"), None)` → `"\\sqrt{x}"`
- [ ] Teste LaTeX: `Fraction(Variable("a"), Variable("b"))` → `"\\frac{a}{b}"`
- [ ] Teste LaTeX: `Integral { lower: 0, upper: 1, body: x², var: "x" }` → `"\\int_{0}^{1} x^{2} \\, dx"`
- [ ] Teste LaTeX: `Constant(Pi)` → `"\\pi"`
- [ ] Teste: nested expressions renderizam correctamente em ambos os formatos
- [ ] Teste: round-trip: NL → AST → LaTeX → visually correct
- [ ] Compilável para WASM
- [ ] Latência de render < 1ms para expressões complexas

**Estimativa:** 2 sessões DDI (~2h)

---

### US-MS-008: MathML Renderer

- **Como** developer web
- **Quero** output em MathML para acessibilidade e rendering nativo no browser
- **Para que** screen readers e browsers modernos possam renderizar fórmulas sem JavaScript

**DDI Intent:**
```
Quero um MathMLRenderer que implemente o trait Renderer.
Ele deve:
- Produzir MathML válido (namespace http://www.w3.org/1998/Math/MathML)
- Usar elementos semânticos: <msup>, <mfrac>, <msqrt>, <mrow>, <mi>, <mn>, <mo>
- Suportar todos os nós do MathAST
- Output é uma String com o XML do MathML (sem <math> wrapper — o chamador adiciona)
```

**Test Criteria:**
- [ ] Teste: `Number(42)` → `<mn>42</mn>`
- [ ] Teste: `Variable("x")` → `<mi>x</mi>`
- [ ] Teste: `Power(x, 2)` → `<msup><mi>x</mi><mn>2</mn></msup>`
- [ ] Teste: `Fraction(a, b)` → `<mfrac><mi>a</mi><mi>b</mi></mfrac>`
- [ ] Teste: `Root(x, None)` → `<msqrt><mi>x</mi></msqrt>`
- [ ] Teste: `Function(Sin, x)` → `<mrow><mi>sin</mi><mo>(</mo><mi>x</mi><mo>)</mo></mrow>`
- [ ] Teste: nested expression → valid nested MathML
- [ ] Teste: output é well-formed XML
- [ ] Compilável para WASM

**Estimativa:** 2 sessões DDI (~1h30)

---

## Sprint 13 (Integration): WASM + Scriptorium

### US-MS-009: WASM Bindings (wasm-pack)

- **Como** developer web
- **Quero** usar MathScribe no browser via JavaScript
- **Para que** o Scriptorium web e Arca School possam converter fórmulas client-side

**DDI Intent:**
```
Quero bindings WASM no crate mathscribe-wasm.
Funções exportadas via wasm-bindgen:
- parse(input: &str, language: Option<&str>) → JsValue (serializado: { latex, unicode, mathml, ast })
- render_latex(input: &str) → String
- render_unicode(input: &str) → String
- render_mathml(input: &str) → String
- detect_language(input: &str) → String ("pt", "en", "es")

Build:
- wasm-pack build --target web (para importar como ES module)
- Size budget: < 200KB gzipped
- Funciona em Chrome 86+, Firefox 111+, Safari 15.2+
```

**Test Criteria:**
- [ ] Teste: `wasm-pack build --target web` compila sem erros
- [ ] Teste: `parse("x ao quadrado", None)` retorna { latex: "x^{2}", unicode: "x²" }
- [ ] Teste: `render_latex("x squared")` → `"x^{2}"`
- [ ] Teste: `render_unicode("x al cuadrado")` → `"x²"`
- [ ] Teste: `detect_language("raiz quadrada")` → `"pt"`
- [ ] Teste: WASM binary < 200KB gzipped
- [ ] wasm-bindgen-test passa em headless Chrome

**Estimativa:** 2 sessões DDI (~2h)

---

### US-MS-010: Scriptorium Plugin — MathParser Implementation

- **Como** developer
- **Quero** que MathScribe implemente o trait MathParser do Scriptorium
- **Para que** o Scriptorium possa usar MathScribe como drop-in replacement do SimpleMathParser

**DDI Intent:**
```
Quero criar um módulo de integração com o Scriptorium.
Ele deve:
- Definir struct MathScribeParser que implementa o trait MathParser
  (trait definido no scriptorium-core US-039)
- MathScribeParser.parse(input) deve:
  1. Tokenizar o input
  2. Detectar língua
  3. Fazer parse com a grammar correcta
  4. Renderizar em Unicode e LaTeX
  5. Retornar MathResult { source_nl, rendered_unicode, rendered_latex }
- Ser configurável: quais línguas activar, language override
- Ser instanciável sem parâmetros (defaults sensatos)
- Feature flag: "scriptorium" no Cargo.toml — só compila se activado

NOTA: Esta US depende do trait MathParser estar definido no scriptorium-core.
Se o trait ainda não existir, definir uma versão local compatível.
```

**Test Criteria:**
- [ ] Teste: `MathScribeParser::default()` cria parser com 3 gramáticas (PT, EN, ES)
- [ ] Teste: `parser.parse("x ao quadrado")` → MathResult com unicode "x²" e latex "x^{2}"
- [ ] Teste: `parser.parse("square root of x")` → MathResult com unicode "√x" e latex "\\sqrt{x}"
- [ ] Teste: `parser.parse("x al cuadrado")` → MathResult correctamente
- [ ] Teste: `parser.parse("unknown gibberish")` → MathResult com source_nl preservado, rendered = input
- [ ] Teste: latência < 16ms para expressões complexas (60fps budget)
- [ ] Feature flag "scriptorium" compila correctamente
- [ ] Sem feature flag → módulo de integração não compila (zero overhead)

**Cross-Project Dependency (Scriptorium):**
- **Requer Scriptorium US-039** (MathParser trait definido no scriptorium-core)
- Se US-039 ainda não estiver implementada, definir trait local compatível com nota TODO

**Estimativa:** 1 sessão DDI (~1h)

---

## Sprint & Dependency Summary

```
Sprint 10 (Wk 23-24): FOUNDATION
  US-MS-001: Repo + ADR ──────────┐
  US-MS-002: MathAST ─────────────┤── US-MS-002 depende de US-MS-001
  US-MS-003: Tokenizer ───────────┘── US-MS-003 depende de US-MS-001

Sprint 11 (Wk 25-26): GRAMMARS
  US-MS-004: Portuguese Grammar ──┐
  US-MS-005: English Grammar ─────┤── Todos dependem de US-MS-002 + US-MS-003
  US-MS-006: Spanish Grammar ─────┘── Podem ser paralelos entre si

Sprint 12 (Wk 27-28): RENDERERS
  US-MS-007: LaTeX + Unicode ─────┐── Depende de US-MS-002 (MathAST)
  US-MS-008: MathML ──────────────┘── Depende de US-MS-002

Sprint 13 (Wk 29-30): INTEGRATION
  US-MS-009: WASM bindings ───────── Depende de US-MS-007 + grammars
  US-MS-010: Scriptorium Plugin ──── Depende de US-MS-007 + Scriptorium US-039 ⚠️
```

### Cross-Project Dependency Map

```
┌─────────────────────────────┐        ┌─────────────────────────────┐
│       MATHSCRIBE            │        │        SCRIPTORIUM          │
│                             │        │                             │
│  US-MS-010 (Plugin) ───────┼───────►│  US-039 (MathParser trait)  │
│    implements MathParser    │ trait  │  defines MathParser trait    │
│                             │        │                             │
│  US-MS-009 (WASM) ─────────┼───────►│  Scriptorium Web (WASM)    │
│    JS module for browser    │ import │  loads mathscribe-wasm      │
│                             │        │                             │
│  US-MS-007 (Renderers) ────┼───────►│  US-037 (Action Bar)       │
│    Unicode for live preview │ render │  MathScribe mode preview    │
│                             │        │                             │
└─────────────────────────────┘        └─────────────────────────────┘
```

---

## Recommended Execution Order

```
US-MS-001 (repo) → US-MS-002 (AST) → US-MS-003 (tokenizer)
→ US-MS-004 (PT grammar) → US-MS-005 (EN) → US-MS-006 (ES)
→ US-MS-007 (LaTeX + Unicode) → US-MS-008 (MathML)
→ US-MS-009 (WASM) → US-MS-010 (Scriptorium plugin)
```

US-MS-004/005/006 podem correr em paralelo.
US-MS-007 e US-MS-008 podem correr em paralelo.
US-MS-010 depende do trait MathParser existir no Scriptorium.

---

## Q2-Q3 2026 Deliverables

- [ ] MathAST: expression tree with all math types (US-MS-002)
- [ ] Tokenizer: NL + math → Token stream with language detection (US-MS-003)
- [ ] Portuguese Grammar: 15+ patterns (US-MS-004)
- [ ] English Grammar: 15+ patterns (US-MS-005)
- [ ] Spanish Grammar: 15+ patterns (US-MS-006)
- [ ] LaTeX Renderer: standard LaTeX output (US-MS-007)
- [ ] Unicode Renderer: fast preview with superscripts/symbols (US-MS-007)
- [ ] MathML Renderer: accessible web output (US-MS-008)
- [ ] WASM bindings: < 200KB, works in modern browsers (US-MS-009)
- [ ] Scriptorium Plugin: implements MathParser trait (US-MS-010) ⚠️ Scriptorium US-039
- [ ] ADR-001: MathScribe Architecture
- [ ] Devlog #7: "Parsing natural language math — deterministic NL→LaTeX in Rust"
- [ ] **mathscribe v0.1 published**

---

## Claude Code Prompt Template (Q2-Q3 2026)

```
Act as a Senior Rust Engineer. We are starting a new task following
the @DDI-INSTRUCTIONS.md protocol and adhering to
@docs/ADR-001-mathscribe-architecture.md.

Context:
- Project: MathScribe (standalone NL→formula engine, Rust)
- Quarter: Q2-Q3 2026 — NL→Formula Engine
- Task: US-MS-0XX — [US Name]
- Architecture: Core (WASM-compatible) + WASM bindings
- Languages: Portuguese + English + Spanish
- No LLM: deterministic rule-based parsing

Step 1 (Intent Analysis):
Review the ADR. I will provide my intent.
Do not write any implementation code yet.

Step 2 (Test-First):
Once I provide the intent, your first output must be the test cases.

Step 3 (Incremental Implementation):
Only after I approve the tests, proceed in blocks of no more than 50 lines.

Ready? Let me know so I can provide my Intent for this task.
```

---

**Last Updated:** Mar 2026
