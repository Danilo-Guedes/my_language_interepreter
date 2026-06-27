# GuedzLang

A tree-walking interpreter for **GuedzLang**, written in Rust. Based on the
Monkey language from *Writing An Interpreter In Go* (Thorsten Ball), ported to
Rust using a TDD development approach.

It is a small but complete dynamically-typed language: integers, booleans,
strings, arrays, and hashes; first-class functions with **closures** and
**recursion**; `if`/`else` expressions; a set of built-in functions; and `//`
line comments.

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

## Language tour

GuedzLang is dynamically typed and **expression-oriented** — almost everything
produces a value, including `if`. The last expression in a function body is its
return value, so `return` is optional. Line comments start with `//`.

```guedz
// Variables and arithmetic
let age = 30;
let next = age + 1;                  // => 31

// Functions are first-class values; the last expression is returned
let double = fn(x) { x * 2 };
double(21);                          // => 42

// `if` is an expression — it evaluates to a value
let max = fn(a, b) { if (a > b) { a } else { b } };
max(7, 3);                           // => 7

// Closures: an inner function captures its surrounding scope
let newAdder = fn(x) { fn(y) { x + y } };
let addTwo = newAdder(2);
addTwo(10);                          // => 12

// Recursion: a let-bound function can call itself
let fib = fn(n) { if (n < 2) { n } else { fib(n - 1) + fib(n - 2) } };
fib(10);                             // => 55

// Strings
"Hello" + " " + "World!";            // => Hello World!
len("hello");                        // => 5

// Arrays — heterogeneous, zero-indexed; out-of-bounds yields null
let xs = [1, "two", true];
xs[0];                               // => 1
xs[99];                              // => null

// Hashes — keys may be integers, booleans, or strings
let user = {"name": "Ada", "age": 36};
user["name"];                        // => Ada
user["missing"];                     // => null
```

> The `// =>` annotations show the value each line evaluates to; they are
> ordinary comments and have no effect when run.

## Built-in functions

A handful of built-ins are always in scope. They live in `builtins.rs` and are
seeded into the global `Environment` at startup:

| Function       | Description                                            | Example                         |
| -------------- | ------------------------------------------------------ | ------------------------------- |
| `len(x)`       | Length of a string or array                            | `len([1, 2, 3])` → `3`          |
| `first(arr)`   | First element, or `null` if the array is empty         | `first([10, 20])` → `10`        |
| `last(arr)`    | Last element, or `null` if the array is empty          | `last([10, 20])` → `20`         |
| `rest(arr)`    | A **new** array with everything but the first element  | `rest([1, 2, 3])` → `[2, 3]`    |
| `push(arr, x)` | A **new** array with `x` appended (original unchanged) | `push([1, 2], 3)` → `[1, 2, 3]` |
| `log(...)`     | Prints each argument on its own line; returns `null`   | `log("hi")`                     |

`rest` and `push` are non-mutating — they return fresh arrays instead of
modifying their input, which keeps GuedzLang's values immutable.

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
| `lib.rs`       | Library crate root — declares the modules; the public API the binary and `tests/` build against. |
| `main.rs`      | Thin binary entry point — wires up I/O and launches the REPL.                            |
| `lexer.rs`     | Reads the raw source string and produces **tokens**.                                     |
| `token.rs`     | Defines the `Token` type. Pure data — no logic.                                          |
| `parser.rs`    | Consumes tokens from the lexer and builds the **AST** (handles precedence, grouping).    |
| `ast.rs`       | Defines the AST **node** types — the statements and expressions of the language.         |
| `evaluator.rs` | The tree-walking **evaluator**. Recursively walks the AST and produces `Object`s.        |
| `object.rs`    | Defines runtime **values** (`Object`) **and** the `Environment` (variable scope).        |
| `builtins.rs`  | The built-in functions (`len`, `first`, `push`, …) seeded into the global scope.          |
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

1. **`Object`** — the runtime values the evaluator produces: `Integer`,
   `Boolean`, `StringObj`, `Array`, `HashObj`, `Func`, `Builtin`,
   `ReturnValue`, `Error`, and `Null`.
2. **`Environment`** — the symbol table mapping variable names to values
   (`let x = 5` lives here). Each scope holds an `outer` link, so an inner scope
   can see variables from the scopes enclosing it — that chain is what makes
   nested scopes and closures work.

#### Why `Environment` is an `Rc<RefCell<Environment>>`

Scopes are **shared, not copied**. An environment is held as
`Rc<RefCell<Environment>>` — a reference-counted, interior-mutable handle — and a
closure captures a *clone of that handle* (a pointer to the same scope), not a
snapshot of its contents.

This is what makes recursion work. In:

```guedz
let fib = fn(n) { if (n < 2) { n } else { fib(n - 1) + fib(n - 2) } };
```

the function value captures a pointer to the scope that `let` is *about to*
insert `fib` into. By the time `fib` runs and looks itself up, the name is
already present in the very scope it captured. With by-value environment copies
the closure would hold a stale snapshot taken *before* the binding existed, and
`fib` would fail with `identifier not found: fib`. The shared handle removes that
ordering trap, so closures and recursion both fall out of the same mechanism.

## Tests

```bash
cargo test
```

Unit tests live alongside the code (`#[cfg(test)]` modules in each file); the
`tests/` directory holds end-to-end tests that drive the interpreter through its
public library API — the same way an external program embedding it would.
