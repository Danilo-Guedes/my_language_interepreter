# GuedzLang

A tree-walking interpreter for **GuedzLang**, written in Rust. Based on the
Monkey language from *Writing An Interpreter In Go* (Thorsten Ball), ported to
Rust.

## Running it

```bash
cargo run
```

This starts the REPL:

```
>> let add = fn(x, y) { x + y };
>> add(2, 3)
5
```

## How it works — the mental model

Source code flows through four stages. Each stage transforms its input into a
richer representation, and the next stage only ever talks to the previous one's
output:

```
source code (String)
      │  lexer.rs        — scan characters into tokens
      ▼
   tokens                ← token.rs defines the Token type
      │  parser.rs       — consume tokens, build the tree
      ▼
    AST                  ← ast.rs defines the node types
      │  evaluator.rs    — walk the tree and run it
      ▼
   Objects               ← object.rs defines values + Environment (scope)
      │  repl.rs         — print the result
      ▼
   output
```

One sentence to remember the split: **the lexer and parser figure out *what you
wrote*; the evaluator decides *what it means*; `object.rs` holds the *results
and the variables*.**

## File-by-file responsibilities

| File           | Job                                                                                      |
| -------------- | ---------------------------------------------------------------------------------------- |
| `main.rs`      | Entry point. Declares the modules and launches the REPL.                                 |
| `lexer.rs`     | Reads the raw source string and produces **tokens**.                                     |
| `token.rs`     | Defines the `Token` type. Pure data — no logic.                                          |
| `parser.rs`    | Consumes tokens from the lexer and builds the **AST** (handles precedence, grouping).    |
| `ast.rs`       | Defines the AST **node** types — the statements and expressions of the language.         |
| `evaluator.rs` | The tree-walking **evaluator**. Recursively walks the AST and produces `Object`s.        |
| `object.rs`    | Defines runtime **values** (`Object`) **and** the `Environment` (variable scope).        |
| `repl.rs`      | The Read–Eval–Print Loop. Wires lexer → parser → evaluator → print together.             |

### Tokens vs. AST nodes — why both exist

This is the distinction worth internalizing:

- A **token** is *flat*: `Token::Plus`, `Token::Int(5)`. No structure, no nesting.
- An **AST node** is *structured and nested*: an infix expression *contains* a
  left expression, an operator, and a right expression — each of which may
  contain more nodes.

The parser's entire purpose is that promotion: flat tokens → nested tree.
`5 + 5 * 2` is just five tokens in a row, but the AST nests the `*` *inside* the
`+` to capture precedence. That tree shape is the value `ast.rs` adds.

### Parser vs. evaluator

The parser only describes *what the code says*. The evaluator decides *what it
does*. `evaluator.rs` is the heart of the interpreter:
`eval_program` → `eval_statement` → `eval_expression`, recursing down the tree
until it bottoms out in concrete `Object` values.

### Object and Environment

`object.rs` has two responsibilities:

1. **`Object`** — the runtime values: `Integer`, `Boolean`, `Func`,
   `ReturnValue`, `Error`, `Null`.
2. **`Environment`** — the symbol table mapping variable names to values
   (`let x = 5` lives here). It has an `outer` chain so inner scopes can see
   outer variables, which is what makes nested scopes and closures work.

## Tests

```bash
cargo test
```
