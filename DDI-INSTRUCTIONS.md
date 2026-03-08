# DDI — Desenvolvimento Dirigido por Intenção

> **Protocolo de desenvolvimento para MathScribe.**
> Cada sessão de Claude Code deve começar por ler este ficheiro.

---

## 1. O Ciclo DDI (4 passos)

### Passo 1 — INTENÇÃO (Humano)
O humano descreve em linguagem natural **o que quer**, sem especificar como.

### Passo 2 — TESTE (AI)
A AI escreve os **testes primeiro** que capturam a intenção.
O humano revê: "estes testes capturam o que eu pedi?"

### Passo 3 — IMPLEMENTAÇÃO (AI)
Só após aprovação dos testes, a AI implementa em blocos de **20-50 linhas**.
Cada bloco é explicado antes de ser escrito.

### Passo 4 — REVISÃO GUIADA (Humano)
O humano lê o código, pergunta o que não entende.
Commit só acontece quando o humano entendeu tudo.

---

## 2. Regras Operacionais

### Tamanho e Escopo
- **Máximo 50 linhas** por bloco de implementação
- **Um conceito por vez**: uma struct, uma função, um trait
- Se o bloco excede 50 linhas → dividir em sub-tarefas

### Testes
- Testes **sempre antes** do código
- Naming: `test_<língua>_<padrão>` (e.g., `test_pt_power`, `test_en_square_root`)
- Snapshot tests para NL→LaTeX: input/output ficheiros `.txt`
- Cada gramática tem suite separada

### Commits
- Mensagem descreve a **INTENÇÃO**, não o código
- Formato: `feat:`, `fix:`, `refactor:`, `test:`, `chore:`
- Um commit por User Story ou sub-tarefa

### Qualidade
- `cargo clippy --workspace -- -D warnings` sem warnings
- `cargo fmt --all` antes de cada commit
- `cargo build -p mathscribe-core --target wasm32-unknown-unknown` deve passar

---

## 3. Architecture and Context Guidelines

- **Consult ADR:** Before implementing, read `docs/ADR-001-mathscribe-architecture.md` for design decisions.
- **WASM Check:** After every implementation, verify that `mathscribe-core` still compiles to `wasm32-unknown-unknown`. This is a hard constraint.
- **Docstrings:** Use docstrings to record the "intent" and the contract directly within the code.
- **Architectural Inquiry:** Focus explanations on the "Why" (Design Decisions) rather than just the "What" (Syntax).

### ADR References
- **ADR-001:** MathScribe architecture — Pipeline NL→AST→Formula, Grammar trait, Renderer trait, WASM strategy, language detection

---

## 4. MathScribe-Specific Rules

- **Deterministic Parsing:** MathScribe does NOT use LLMs. All conversions are rule-based, deterministic.
- **Grammar Trait:** Every language is a `Grammar` implementation. Never hardcode language-specific patterns.
- **Fallback Graceful:** If a pattern is not recognized, return the input as a `Variable` node. Never panic.
- **Unicode First:** The Unicode renderer is the fast-path for live preview. LaTeX is for export.
- **Extensible:** Adding a new language = implementing `Grammar` trait. No changes to parser or renderer.
- **No Regex:** Use recursive descent parsing. Regex is fragile for nested mathematical expressions.
- **Precision:** Mathematical notation must be correct. `x^2 + 1` must render as `x² + 1`, not `x2 + 1`.

---

## 5. Cross-Project Coordination

MathScribe integrates with Scriptorium via the `MathParser` trait (defined in Scriptorium US-039):

```rust
// This trait is defined in scriptorium-core
trait MathParser {
    fn parse(&self, input: &str) -> MathResult;
}

// MathScribe implements it
struct MathScribeParser { /* ... */ }
impl MathParser for MathScribeParser {
    fn parse(&self, input: &str) -> MathResult {
        // tokenize → parse → render
    }
}
```

The `MathResult` contains: `source_nl`, `rendered_unicode`, `rendered_latex`.

---

### How to use this in your workflow:

1. **Claude Code:** `claude "Read DDI-INSTRUCTIONS.md and CLAUDE.md. Apply the DDI workflow to implement US-MS-0XX."`
2. **Start a session:** Type `DDI: Start US-MS-002` to begin.
