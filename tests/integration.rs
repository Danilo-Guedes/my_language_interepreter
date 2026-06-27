
use guedzlang::evaluator::Evaluator;
use guedzlang::lexer::Lexer;
use guedzlang::object::Object;
use guedzlang::parser::Parser;

/// Lex -> parse -> eval a source string, asserting it parses without errors.
fn run(input: &str) -> Object {
    let lexer = Lexer::new(input);
    let mut parser = Parser::new(lexer);
    let program = parser.parse_program();
    assert!(
        parser.errors().is_empty(),
        "unexpected parser errors: {:?}",
        parser.errors()
    );

    let mut evaluator = Evaluator::new();
    evaluator.eval_program(program)
}

fn expect_integer(input: &str, expected: i64) {
    match run(input) {
        Object::Integer(value) => assert_eq!(value, expected, "input: {input}"),
        other => panic!("expected Integer({expected}), got {other:?} for input: {input}"),
    }
}

#[test]
fn evaluates_arithmetic_with_bindings() {
    expect_integer("let a = 5; let b = 10; a + b * 2;", 25);
}

#[test]
fn recursion_works_end_to_end() {
    // exercises the Rc<RefCell<Environment>> shared-scope design
    expect_integer(
        "let fib = fn(n) { if (n < 2) { n } else { fib(n - 1) + fib(n - 2) } }; fib(10);",
        55,
    );
}

#[test]
fn closures_capture_their_scope() {
    expect_integer(
        "let adder = fn(x) { fn(y) { x + y } }; let addTwo = adder(2); addTwo(40);",
        42,
    );
}

#[test]
fn strings_and_builtins() {
    expect_integer(r#"let s = "Hello" + " " + "World!"; len(s);"#, 12);
}

#[test]
fn arrays_with_push_and_indexing() {
    expect_integer("let xs = push([1, 2], 3); xs[2];", 3);
}

#[test]
fn line_comments_are_ignored() {
    expect_integer("let x = 41; // this is a comment\n x + 1;", 42);
}
