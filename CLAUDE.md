# CLAUDE.md — MathScribe

> Este ficheiro dá contexto ao Claude Code para trabalhar no projecto MathScribe.
> Colocar na raiz do repo `mathscribe/`.

---

## O Projecto

MathScribe é um motor de conversão de linguagem natural para fórmulas matemáticas, escrito em Rust. Aceita input em Português, Inglês e Espanhol e produz output em LaTeX, Unicode e MathML.

O nome combina "Math" com "Scribe" (escriba) — o motor que transcreve a intenção matemática do aluno em notação formal. É o elo entre a escrita natural do ensaio e a precisão da fórmula.

Faz parte do ecossistema Arcasidian:

```
Arcasidian (empresa)
├── arca-api        →  servidor central always-on (Axum + PostgreSQL)
├── slm-engine      →  worker de inferência batch (Candle + GGUF, GPU on-demand)
├── Scriptorium     →  editor de ensaios (desktop + WASM)
├── Spec Machine    →  dataset + ambiguity reduction
├── MathScribe      →  engine NL→fórmula (ESTE PROJECTO)
├── Glossator       →  agente IA (poda, polinização, adubação)
└── Arca School     →  plataforma educacional B2C
```

MathScribe é um projecto **standalone** — NÃO depende do Scriptorium.
O Scriptorium consome MathScribe como dependência (crate ou WASM module).

Repo: https://github.com/DbonesDev/mathscribe

---

## Stack Técnica

- **Linguagem:** Rust (edition 2021)
- **Parser:** Custom recursive descent (sem deps externas de parser)
- **Serialização:** serde + serde_json
- **Unicode:** unicode-segmentation (tokenização correcta de caracteres multi-byte)
- **WASM:** wasm-bindgen + wasm-pack (bindings para browser)
- **CI:** GitHub Actions (fmt + clippy + tests + WASM build)
- **Target WASM:** `wasm32-unknown-unknown` (core) + `wasm-pack build` (wasm crate)

---

## Arquitectura: Pipeline NL→Fórmula

```
┌────────────────────────────────────────────────────────┐
│                  MathScribe Core                        │
│                   (Rust puro)                           │
│                                                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐ │
│  │  Tokenizer   │  │   Parser     │  │  Renderer    │ │
│  │              │  │              │  │              │ │
│  │ NL text →    │  │ Tokens →     │  │ MathAST →    │ │
│  │ Token stream │  │ MathAST      │  │ LaTeX        │ │
│  │              │  │              │  │ Unicode      │ │
│  │ Multi-lang:  │  │ Recursive    │  │ MathML       │ │
│  │ PT, EN, ES   │  │ descent      │  │              │ │
│  └──────────────┘  └──────────────┘  └──────────────┘ │
│                                                        │
│  ┌──────────────────────────────────────────────────┐  │
│  │  Language Registry                                │  │
│  │  • PortugueseGrammar: "x ao quadrado" → Power    │  │
│  │  • EnglishGrammar: "x squared" → Power            │  │
│  │  • SpanishGrammar: "x al cuadrado" → Power        │  │
│  │  • Extensível: trait Grammar para novas línguas    │  │
│  └──────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────┘
         │                              │
   ┌─────┘                        ┌─────┘
   ▼                              ▼
┌───────────┐              ┌───────────────┐
│ WASM      │              │ Scriptorium   │
│ (browser) │              │ Plugin        │
│ via       │              │ (implements   │
│ wasm-     │              │  MathParser   │
│ bindgen   │              │  trait)       │
└───────────┘              └───────────────┘
```

**Pipeline:**
1. **Tokenizer** — Recebe texto NL, detecta língua, produz stream de tokens (Number, Operator, Variable, Keyword)
2. **Parser** — Consome tokens, aplica regras gramaticais da língua detectada, constrói MathAST
3. **Renderer** — Percorre MathAST, produz output no formato desejado (LaTeX, Unicode, MathML)

**Regra fundamental:** `mathscribe-core` é Rust puro SEM dependências de sistema. Deve compilar para `wasm32-unknown-unknown` sem erros.

---

## Estrutura do Repo

```
mathscribe/
├── crates/
│   ├── core/           # Engine pura: tokenizer, parser, AST, renderer, grammars
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── ast.rs          # MathAST: Expression, Operator, Function nodes
│   │       ├── tokenizer.rs    # Tokenizer: NL text → Token stream
│   │       ├── parser.rs       # Parser: Tokens → MathAST (recursive descent)
│   │       ├── renderer.rs     # Renderer trait + LaTeX, Unicode, MathML renderers
│   │       ├── grammar.rs      # Grammar trait + Language detection
│   │       ├── grammars/
│   │       │   ├── mod.rs
│   │       │   ├── portuguese.rs  # PT patterns: "ao quadrado", "raiz de", etc.
│   │       │   ├── english.rs     # EN patterns: "squared", "square root of", etc.
│   │       │   └── spanish.rs     # ES patterns: "al cuadrado", "raíz de", etc.
│   │       └── error.rs        # MathScribeError enum
│   └── wasm/           # WASM bindings (wasm-bindgen)
│       └── src/
│           └── lib.rs
├── docs/
│   ├── ADR-001-mathscribe-architecture.md
│   └── Phase-4-User-Stories.md
├── .adr/
│   └── ADR-001.md      # Symlink ou cópia do ADR em docs/
├── .github/workflows/
│   └── ci.yml
├── Cargo.toml          # Workspace
├── CLAUDE.md           # Este ficheiro
├── DDI-INSTRUCTIONS.md
└── .gitignore
```

---

## Gramáticas Suportadas

### Português
```
"x ao quadrado"          → x²
"x ao cubo"              → x³
"raiz quadrada de x"     → √x
"raiz cúbica de x"       → ∛x
"a sobre b"              → a/b (fracção)
"integral de x"          → ∫x
"integral de a até b"    → ∫ₐᵇ
"soma de i=1 até n"      → Σᵢ₌₁ⁿ
"produto de i=1 até n"   → Πᵢ₌₁ⁿ
"seno de x"              → sin(x)
"limite de x tendendo a 0" → lim_{x→0}
"derivada de f em x"     → df/dx
"x maior ou igual a y"   → x ≥ y
"x pertence a R"         → x ∈ ℝ
```

### English
```
"x squared"              → x²
"x cubed"                → x³
"square root of x"       → √x
"cube root of x"         → ∛x
"a over b"               → a/b
"integral of x"          → ∫x
"sum from i=1 to n"      → Σᵢ₌₁ⁿ
"product from i=1 to n"  → Πᵢ₌₁ⁿ
"sine of x"              → sin(x)
"limit as x approaches 0" → lim_{x→0}
"derivative of f with respect to x" → df/dx
"x greater than or equal to y" → x ≥ y
"x belongs to R"         → x ∈ ℝ
```

### Español
```
"x al cuadrado"          → x²
"x al cubo"              → x³
"raíz cuadrada de x"     → √x
"raíz cúbica de x"       → ∛x
"a sobre b"              → a/b
"integral de x"          → ∫x
"suma de i=1 hasta n"    → Σᵢ₌₁ⁿ
"producto de i=1 hasta n" → Πᵢ₌₁ⁿ
"seno de x"              → sin(x)
"límite de x tendiendo a 0" → lim_{x→0}
"derivada de f respecto a x" → df/dx
"x mayor o igual que y"  → x ≥ y
"x pertenece a R"        → x ∈ ℝ
```

---

## MathAST — O modelo central

```rust
enum MathExpr {
    // Literais
    Number(f64),
    Variable(String),
    Constant(MathConstant),     // Pi, E, Infinity

    // Operações binárias
    Add(Box<MathExpr>, Box<MathExpr>),
    Sub(Box<MathExpr>, Box<MathExpr>),
    Mul(Box<MathExpr>, Box<MathExpr>),
    Div(Box<MathExpr>, Box<MathExpr>),
    Power(Box<MathExpr>, Box<MathExpr>),

    // Funções
    Function(MathFunc, Box<MathExpr>),   // sin, cos, log, ln
    Root(Box<MathExpr>, Option<Box<MathExpr>>),  // √x, ∛x

    // Cálculo
    Integral { body: Box<MathExpr>, var: String, lower: Option<Box<MathExpr>>, upper: Option<Box<MathExpr>> },
    Derivative { body: Box<MathExpr>, var: String, order: u32 },
    Limit { body: Box<MathExpr>, var: String, approaching: Box<MathExpr> },
    Sum { body: Box<MathExpr>, var: String, lower: Box<MathExpr>, upper: Box<MathExpr> },
    Product { body: Box<MathExpr>, var: String, lower: Box<MathExpr>, upper: Box<MathExpr> },

    // Relações
    Equals(Box<MathExpr>, Box<MathExpr>),
    LessThan(Box<MathExpr>, Box<MathExpr>),
    GreaterThan(Box<MathExpr>, Box<MathExpr>),
    LessOrEqual(Box<MathExpr>, Box<MathExpr>),
    GreaterOrEqual(Box<MathExpr>, Box<MathExpr>),
    ElementOf(Box<MathExpr>, Box<MathExpr>),

    // Agrupamento
    Parenthesized(Box<MathExpr>),
    Fraction(Box<MathExpr>, Box<MathExpr>),
}
```

---

## GitHub Workflow — Issues como Fonte de Verdade

As User Stories deste projecto vivem como **GitHub Issues** no repo `DbonesDev/mathscribe`, sincronizadas com o **GitHub Project** "Arcasidian Roadmap".

### Antes de começar a trabalhar

```bash
# Ver todas as US abertas deste repo, ordenadas por sprint
gh issue list --repo DbonesDev/mathscribe --label user-story --state open --json number,title,labels --jq '.[] | "\(.number)\t\(.title)\t\(.labels | map(.name) | join(","))"'

# Ver detalhes de uma US específica
gh issue view <number> --repo DbonesDev/mathscribe
```

**Regra:** Sempre consultar as issues abertas antes de iniciar trabalho. Respeitar a ordem de sprint e dependências descritas no corpo da issue.

### Durante o desenvolvimento

- Criar branch `feature/US-MS-<ID>-<descrição>` a partir da issue
- Seguir o checklist DDI que está no corpo de cada issue
- Referenciar a issue nos commits: `feat: US-MS-001 repo + ADR (#<number>)`

### Ao concluir uma US

```bash
# Fechar a issue quando todos os critérios DDI estiverem cumpridos
gh issue close <number> --repo DbonesDev/mathscribe --comment "✅ DDI completo: testes passam, code review feito, docs actualizados."
```

### Ver roadmap completo (todos os repos)

```bash
# Listar todas as issues do projecto Arcasidian Roadmap
gh project item-list --owner @me --format json | head -50
```

---

## Workflow de Desenvolvimento: DDI

Este projecto usa **Desenvolvimento Dirigido por Intenção (DDI)**. O ciclo é:

1. **INTENÇÃO** — Descrevo em português o que quero
2. **TESTE** — Tu escreves o teste primeiro, eu valido se captura a intenção
3. **IMPLEMENTAÇÃO** — Tu escreves o código que faz o teste passar (blocos de 20-50 linhas)
4. **REVISÃO** — Eu leio, pergunto o que não entendo, commit só quando entendi

### Regras DDI

- Teste SEMPRE antes do código
- Blocos pequenos: uma feature, um componente, uma struct por vez
- Nunca gerar mais de 50 linhas sem explicar
- Mensagens de commit descrevem a INTENÇÃO, não o código
- Formato de commit: `feat:`, `fix:`, `refactor:`, `test:`, `chore:`

---

## Comandos Essenciais

```bash
# Compilar todo o workspace
cargo build --workspace

# Rodar todos os testes
cargo test --workspace

# Lint (zero warnings é o objectivo)
cargo clippy --workspace -- -D warnings

# Formatar código
cargo fmt --all

# Verificar compilação WASM do core (CRÍTICO — deve sempre funcionar)
cargo build -p mathscribe-core --target wasm32-unknown-unknown

# Build WASM pack (para browser)
cd crates/wasm && wasm-pack build --target web
```

---

## Estado Actual (Mar 2026)

**Fase 4 — Sprint 0: Scaffolding**

### A fazer primeiro:
- US-MS-001: Criar repo + ADR de arquitectura
- US-MS-002: MathAST + MathExpr enum
- US-MS-003: Tokenizer (NL → tokens)
- US-MS-004: Parser PT (tokens → MathAST)

---

## Convenções de Código

### Separação Core / WASM
```
mathscribe-core    → Rust puro, zero deps de sistema, compila para WASM
mathscribe-wasm    → wasm-bindgen, chama core, exporta para JS
```

O WASM crate NUNCA contém lógica de parsing. Chama métodos do core.

### Testes
- Testes unitários: `#[test]`
- Testes de snapshot: input NL → output LaTeX (ficheiros .txt)
- Naming: `test_<língua>_<padrão>` (e.g., `test_pt_power`, `test_en_integral`)
- Cada gramática tem a sua suite de testes

### Parser
- Recursive descent sem backtracking
- Cada regra gramatical é uma fn separada
- Erros devolvem posição + sugestão: "Did you mean 'x ao quadrado'?"
- Fallback: se não reconhece padrão → retorna input raw como Variable

### Renderer
- Trait `Renderer` com método `render(&self, ast: &MathExpr) -> String`
- Implementações: `LatexRenderer`, `UnicodeRenderer`, `MathMLRenderer`
- LaTeX usa convenções standard: `\frac{}{}`, `\sqrt{}`, `\int`, `\sum`

---

## User Stories (Q2-Q3 2026 — 10 US com prefixo MS)

Ver `docs/Phase-4-User-Stories.md` para detalhes completos.

### Sprint 10 (Foundation): US-MS-001 → US-MS-003
- US-MS-001: Repo + ADR
- US-MS-002: MathAST (Expression tree + serde)
- US-MS-003: Tokenizer (NL text → Token stream)

### Sprint 11 (Grammars): US-MS-004 → US-MS-006
- US-MS-004: Portuguese Grammar
- US-MS-005: English Grammar
- US-MS-006: Spanish Grammar

### Sprint 12 (Renderers): US-MS-007 → US-MS-008
- US-MS-007: LaTeX + Unicode Renderers
- US-MS-008: MathML Renderer

### Sprint 13 (Integration): US-MS-009 → US-MS-010
- US-MS-009: WASM bindings (wasm-pack)
- US-MS-010: Scriptorium Plugin (implements MathParser trait)

---

## O que NÃO fazer

- NÃO adicionar deps de sistema ao core (deve compilar para WASM)
- NÃO usar regex para parsing (recursive descent é mais robusto e extensível)
- NÃO depender de LLMs para a conversão (parsing puro determinístico)
- NÃO adicionar dependências sem justificação
- NÃO gerar mais de 50 linhas sem explicar
- NÃO fazer commit de secrets
- NÃO pular testes
- NÃO quebrar a compilação WASM do core
- NÃO hardcodar padrões de língua (usar o Grammar trait)

---

## Conexão com o Ecossistema

| Componente | Integração com MathScribe |
|------------|--------------------------|
| **Scriptorium (Q3 2026)** | Plugin: implements `MathParser` trait, substitui `SimpleMathParser` (US-039) |
| **Scriptorium Action Bar** | Modo MathScribe: preview em tempo real via Unicode renderer |
| **slm-engine (futuro)** | Jobs `MathscribeConvert`: MathScribe como fallback se SLM falha |
| **Arca School (futuro)** | WASM module embeddado no editor web do aluno |

---

## Documentação de Referência

Ficheiros neste repo (`docs/`):

- `ADR-001-mathscribe-architecture.md` — Pipeline NL→AST→Formula, Grammar trait, Renderer trait, WASM strategy
- `Phase-4-User-Stories.md` — 10 US: Foundation, Grammars (PT/EN/ES), Renderers, Integration

Cross-project:

- Scriptorium `docs/Phase-3-User-Stories.md` US-039 — MathScribe Preview (defines `MathParser` trait that this project implements)
- Scriptorium `CLAUDE.md` — Action Bar MathScribe mode
