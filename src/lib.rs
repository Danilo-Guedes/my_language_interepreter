//! GuedzLang — a tree-walking interpreter for the Monkey language, in Rust.
//!
//! This crate exposes the interpreter as a library: lex a source string into
//! tokens, parse those into an AST, then evaluate the AST into runtime
//! [`object::Object`] values. The `guedzlang` binary is just one consumer of
//! this API (the REPL); other programs can embed the interpreter the same way
//! the integration tests in `tests/` do.

pub mod ast;
pub mod builtins;
pub mod evaluator;
pub mod lexer;
pub mod object;
pub mod parser;
pub mod repl;
pub mod token;
