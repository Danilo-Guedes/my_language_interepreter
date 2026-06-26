use crate::token::Token;

use std::fmt;

pub trait Node {
    fn token_literal(&self) -> String;
}

#[derive(Debug, Clone)]
pub enum StatementNode {
    Let(LetStatement),
    Return(ReturnStatement),
    Expression(ExpressionStatement),
    Block(BlockStatement),
}

impl Node for StatementNode {
    fn token_literal(&self) -> String {
        match self {
            Self::Let(let_stmt) => let_stmt.token_literal(),
            Self::Return(return_stmt) => return_stmt.token_literal(),
            Self::Expression(expression_stmt) => expression_stmt.token_literal(),
            Self::Block(block_stmt) => block_stmt.token_literal(),
        }
    }
}

impl fmt::Display for StatementNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Let(let_stmt) => write!(f, "{}", let_stmt),
            Self::Return(return_stmt) => write!(f, "{}", return_stmt),
            Self::Expression(expression_stmt) => write!(f, "{}", expression_stmt),
            Self::Block(block_stmt) => write!(f, "{}", block_stmt),
        }
    }
}

#[derive(Debug, Default, Clone)]

pub enum ExpressionNode {
    #[default]
    None,
    IdentifierNode(Identifier),
    Integer(IntegerLiteral),
    Prefix(PrefixExpression),
    Infix(InfixExpression),
    BooleanNode(Boolean),
    IfExpressionNode(IfExpression),
    Function(FunctionLiteral),
    Call(CallExpression),
    StringExp(StringLiteral),
    Array(ArrayLiteral),
    Index(IndexExpression),
    Hash(HashLiteral),
}

impl Node for ExpressionNode {
    fn token_literal(&self) -> String {
        match self {
            Self::IdentifierNode(identifirer) => identifirer.token_literal(),
            Self::Integer(integer) => integer.token_literal(),
            Self::Prefix(prefix_expression) => prefix_expression.token_literal(),
            Self::Infix(infix_expression) => infix_expression.token_literal(),
            Self::BooleanNode(boolean) => boolean.token_literal(),
            Self::IfExpressionNode(if_expression) => if_expression.token_literal(),
            Self::Function(function) => function.token_literal(),
            Self::Call(call_expression) => call_expression.token_literal(),
            Self::StringExp(string_literal) => string_literal.token_literal(),
            Self::Array(array_literal) => array_literal.token_literal(),
            Self::Index(idx_exp) => idx_exp.token_literal(),
            Self::Hash(hash_literal) => hash_literal.token_literal(),
            Self::None => String::new(),
        }
    }
}

impl fmt::Display for ExpressionNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IdentifierNode(identifier) => write!(f, "{}", identifier),
            Self::Integer(integer) => write!(f, "{}", integer),
            Self::Prefix(prefix_expression) => write!(f, "{}", prefix_expression),
            Self::Infix(infix_expression) => write!(f, "{}", infix_expression),
            Self::BooleanNode(boolean) => write!(f, "{}", boolean),
            Self::IfExpressionNode(if_expression) => write!(f, "{}", if_expression),
            Self::Function(function) => write!(f, "{}", function),
            Self::Call(call_expression) => write!(f, "{}", call_expression),
            Self::StringExp(string_literal) => write!(f, "{}", string_literal),
            Self::Array(array_literal) => write!(f, "{}", array_literal),
            Self::Index(idx_exp) => write!(f, "{}", idx_exp),
            Self::Hash(hash_literal) => write!(f, "{}", hash_literal),
            Self::None => write!(f, ""),
        }
    }
}

pub struct Program {
    pub statements: Vec<StatementNode>,
}

impl Node for Program {
    fn token_literal(&self) -> String {
        if !self.statements.is_empty() {
            match &self.statements[0] {
                StatementNode::Let(let_stmt) => let_stmt.token_literal(),
                StatementNode::Return(return_stmt) => return_stmt.token_literal(),
                StatementNode::Expression(expression_stmt) => expression_stmt.token_literal(),
                StatementNode::Block(block_stmt) => block_stmt.token_literal(),
            }
        } else {
            String::new()
        }
    }
}

impl fmt::Display for Program {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();
        for statement in self.statements.as_slice() {
            out.push_str(statement.to_string().as_str());
        }
        write!(f, "{}", out)
    }
}

#[derive(Debug, Default, Clone)]

pub struct LetStatement {
    pub token: Token,
    pub name: Identifier,
    pub value: Option<ExpressionNode>,
}

impl Node for LetStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}

impl fmt::Display for LetStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();
        out.push_str(self.token_literal().as_str());
        out.push(' ');
        out.push_str(self.name.to_string().as_str());
        out.push_str(" = ");
        if let Some(value) = &self.value {
            out.push_str(value.to_string().as_str());
        }
        out.push(';');
        write!(f, "{}", out)
    }
}

#[derive(Debug, Default, Clone)]

pub struct Identifier {
    pub token: Token,
    pub value: String,
}

impl Node for Identifier {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}

impl fmt::Display for Identifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

#[derive(Debug, Default, Clone)]
pub struct ReturnStatement {
    pub token: Token,
    pub return_value: Option<ExpressionNode>,
}

impl Node for ReturnStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}

impl fmt::Display for ReturnStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();
        out.push_str(self.token_literal().as_str());
        out.push(' ');
        if let Some(return_value) = &self.return_value {
            out.push_str(return_value.to_string().as_str());
        }
        out.push(';');
        write!(f, "{}", out)
    }
}

#[derive(Debug, Default, Clone)]
pub struct ExpressionStatement {
    pub token: Token,
    pub expression: Option<ExpressionNode>,
}

impl Node for ExpressionStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}

impl fmt::Display for ExpressionStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(expression) = &self.expression {
            return write!(f, "{}", expression);
        }
        write!(f, "")
    }
}

#[derive(Debug, Clone)]
pub struct IntegerLiteral {
    pub token: Token,
    pub value: i64,
}

impl Node for IntegerLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}

impl fmt::Display for IntegerLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.token_literal())
    }
}

#[derive(Debug, Default, Clone)]
pub struct PrefixExpression {
    pub token: Token,
    pub operator: String,
    pub right: Box<ExpressionNode>,
}

impl Node for PrefixExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}

impl fmt::Display for PrefixExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();
        out.push('(');
        out.push_str(self.operator.as_str());
        out.push_str(self.right.to_string().as_str());
        out.push(')');
        write!(f, "{}", out)
    }
}
#[derive(Debug, Default, Clone)]
pub struct InfixExpression {
    pub token: Token,
    pub left: Box<ExpressionNode>,
    pub operator: String,
    pub right: Box<ExpressionNode>,
}

impl Node for InfixExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}

impl fmt::Display for InfixExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();
        out.push('(');
        out.push_str(self.left.to_string().as_str());
        out.push_str(format!(" {} ", self.operator).as_str());
        out.push_str(self.right.to_string().as_str());
        out.push(')');
        write!(f, "{}", out)
    }
}

#[derive(Debug, Clone)]
pub struct Boolean {
    pub token: Token,
    pub value: bool,
}

impl Node for Boolean {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}

impl fmt::Display for Boolean {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.token_literal())
    }
}
#[derive(Debug, Default, Clone)]
pub struct IfExpression {
    pub token: Token,
    pub condition: Box<ExpressionNode>,
    pub consequence: BlockStatement,
    pub alternative: Option<BlockStatement>,
}

impl Node for IfExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}

impl fmt::Display for IfExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();
        out.push_str("if");
        out.push_str(self.condition.to_string().as_str());
        out.push(' ');
        out.push_str(self.consequence.to_string().as_str());
        if let Some(alt) = &self.alternative {
            out.push_str("else ");
            out.push_str(alt.to_string().as_str());
        }
        write!(f, "{}", out)
    }
}

#[derive(Debug, Default, Clone)]
pub struct BlockStatement {
    pub token: Token,
    pub statements: Vec<StatementNode>,
}

impl Node for BlockStatement {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}

impl fmt::Display for BlockStatement {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();
        for statement in &self.statements {
            out.push_str(statement.to_string().as_str());
        }
        write!(f, "{}", out)
    }
}
#[derive(Debug, Clone)]
pub struct FunctionLiteral {
    pub token: Token,
    pub parameters: Vec<Identifier>,
    pub body: BlockStatement,
}

impl Node for FunctionLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}

impl fmt::Display for FunctionLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();
        out.push_str(self.token_literal().as_str());
        out.push('(');
        for (i, param) in self.parameters.iter().enumerate() {
            out.push_str(param.to_string().as_str());
            if i != self.parameters.len() - 1 {
                out.push_str(", ");
            }
        }
        out.push(')');
        out.push(' ');
        out.push_str(self.body.to_string().as_str());
        write!(f, "{}", out)
    }
}

#[derive(Debug, Default, Clone)]
pub struct CallExpression {
    pub token: Token,
    pub function: Box<ExpressionNode>,
    pub arguments: Vec<ExpressionNode>,
}

impl Node for CallExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}

impl fmt::Display for CallExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();
        out.push_str(self.function.to_string().as_str());
        out.push('(');
        for (i, arg) in self.arguments.iter().enumerate() {
            out.push_str(arg.to_string().as_str());
            if i != self.arguments.len() - 1 {
                out.push_str(", ");
            }
        }
        out.push(')');
        write!(f, "{}", out)
    }
}

#[derive(Debug, Clone)]
pub struct StringLiteral {
    pub token: Token,
    pub value: String,
}

impl Node for StringLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}

impl fmt::Display for StringLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.token_literal())
    }
}

#[derive(Debug, Clone)]
pub struct ArrayLiteral {
    pub token: Token,
    pub elements: Vec<ExpressionNode>,
}

impl Node for ArrayLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}

impl fmt::Display for ArrayLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();
        let mut elements: Vec<String> = Vec::new();
        for element in &self.elements {
            elements.push(element.to_string());
        }

        out.push('[');
        out.push_str(elements.join(", ").as_str());

        out.push(']');
        write!(f, "{}", out)
    }
}

#[derive(Debug, Clone)]
pub struct IndexExpression {
    pub token: Token,
    pub left: Box<ExpressionNode>,
    pub index: Box<ExpressionNode>,
}

impl Node for IndexExpression {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}

impl fmt::Display for IndexExpression {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();
        out.push('(');
        out.push_str(self.left.to_string().as_str());
        out.push('[');
        out.push_str(self.index.to_string().as_str());
        out.push_str("])");
        write!(f, "{}", out)
    }
}

#[derive(Debug, Clone)]
pub struct HashLiteral {
    pub token: Token, // {}
    pub pairs: Vec<(ExpressionNode, ExpressionNode)>,
}

impl Node for HashLiteral {
    fn token_literal(&self) -> String {
        self.token.literal.clone()
    }
}

impl fmt::Display for HashLiteral {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut out = String::new();
        let mut pairs: Vec<String> = Vec::new();
        for (key, value) in &self.pairs {
            pairs.push(format!("{}: {}", key, value));
        }

        out.push('{');
        out.push_str(pairs.join(", ").as_str());
        out.push('}');
        write!(f, "{}", out)
    }
}

#[cfg(test)]
mod test {
    use super::{ExpressionNode, Identifier, LetStatement, Program, StatementNode};
    use crate::token::{Token, TokenKind};

    #[test]
    fn test_let_statement_print_string() {
        let program = Program {
            statements: vec![StatementNode::Let(LetStatement {
                token: Token {
                    kind: TokenKind::Let,
                    literal: String::from("let"),
                },
                name: Identifier {
                    token: Token {
                        kind: TokenKind::Ident,
                        literal: String::from("myVar"),
                    },
                    value: String::from("myVar"),
                },
                value: Some(ExpressionNode::IdentifierNode(Identifier {
                    token: Token {
                        kind: TokenKind::Ident,
                        literal: String::from("anotherVar"),
                    },
                    value: String::from("anotherVar"),
                })),
            })],
        };

        assert_eq!(
            format!("{}", program),
            "let myVar = anotherVar;",
            "program string representation is wrong. got={}",
            program
        );
    }
}
