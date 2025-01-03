use crate::token::Token;

pub trait Node {
    fn token_literal(&self) -> String;
    fn print_string(&self) -> String;
}

#[derive(Debug)]
pub enum StatementNode {
    Let(LetStatement),
    Return(ReturnStatement),
    Expression(ExpressionStatement),
    Block(BlockStatement),
}

impl Node for StatementNode {
    fn token_literal(&self) -> String {
        return match self {
            Self::Let(let_stmt) => let_stmt.token_literal(),
            Self::Return(return_stmt) => return_stmt.token_literal(),
            Self::Expression(expression_stmt) => expression_stmt.token_literal(),
            Self::Block(block_stmt) => block_stmt.token_literal(),
        };
    }

    fn print_string(&self) -> String {
        return match self {
            Self::Let(let_stmt) => let_stmt.print_string(),
            Self::Return(return_stmt) => return_stmt.print_string(),
            Self::Expression(expression_stmt) => expression_stmt.print_string(),
            Self::Block(block_stmt) => block_stmt.print_string(),
        };
    }
}

#[derive(Debug, Default)]

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
}

impl Node for ExpressionNode {
    fn token_literal(&self) -> String {
        return match self {
            Self::None => String::new(),
            Self::IdentifierNode(identifirer) => identifirer.token_literal(),
            Self::Integer(integer) => integer.token_literal(),
            Self::Prefix(prefix_expression) => prefix_expression.token_literal(),
            Self::Infix(infix_expression) => infix_expression.token_literal(),
            Self::BooleanNode(boolean) => boolean.token_literal(),
            Self::IfExpressionNode(if_expression) => if_expression.token_literal(),
            Self::Function(function) => function.token_literal(),
            Self::Call(call_expression) => call_expression.token_literal(),
        };
    }

    fn print_string(&self) -> String {
        return match self {
            Self::None => String::new(),
            Self::IdentifierNode(identifier) => identifier.print_string(),
            Self::Integer(integer) => integer.print_string(),
            Self::Prefix(prefix_expression) => prefix_expression.print_string(),
            Self::Infix(infix_expression) => infix_expression.print_string(),
            Self::BooleanNode(boolean) => boolean.print_string(),
            Self::IfExpressionNode(if_expression) => if_expression.print_string(),
            Self::Function(function) => function.print_string(),
            Self::Call(call_expression) => call_expression.print_string(),
        };
    }
}

pub struct Program {
    pub statements: Vec<StatementNode>,
}

impl Node for Program {
    fn token_literal(&self) -> String {
        return if self.statements.len() > 0 {
            match &self.statements[0] {
                StatementNode::Let(let_stmt) => let_stmt.token_literal(),
                StatementNode::Return(return_stmt) => return_stmt.token_literal(),
                StatementNode::Expression(expression_stmt) => expression_stmt.token_literal(),
                StatementNode::Block(block_stmt) => block_stmt.token_literal(),
            }
        } else {
            String::new()
        };
    }
    fn print_string(&self) -> String {
        let mut out = String::new();
        for statement in self.statements.as_slice() {
            out.push_str(&statement.print_string().as_str());
        }
        return out;
    }
}

#[derive(Debug, Default)]

pub struct LetStatement {
    pub token: Token,
    pub name: Identifier,
    pub value: Option<ExpressionNode>,
}

impl Node for LetStatement {
    fn token_literal(&self) -> String {
        return self.token.literal.clone();
    }
    fn print_string(&self) -> String {
        let mut out = String::new();
        out.push_str(self.token_literal().as_str());
        out.push_str(" ");
        out.push_str(self.name.print_string().as_str());
        out.push_str(" = ");
        if let Some(value) = &self.value {
            out.push_str(value.print_string().as_str());
        }
        out.push_str(";");
        return out;
    }
}

#[derive(Debug, Default)]

pub struct Identifier {
    pub token: Token,
    pub value: String,
}

impl Node for Identifier {
    fn token_literal(&self) -> String {
        return self.token.literal.clone();
    }
    fn print_string(&self) -> String {
        self.value.clone()
    }
}

#[derive(Debug, Default)]
pub struct ReturnStatement {
    pub token: Token,
    pub return_value: Option<ExpressionNode>,
}

impl Node for ReturnStatement {
    fn token_literal(&self) -> String {
        return self.token.literal.clone();
    }
    fn print_string(&self) -> String {
        let mut out = String::new();
        out.push_str(self.token_literal().as_str());
        out.push_str(" ");
        if let Some(return_value) = &self.return_value {
            out.push_str(return_value.print_string().as_str());
        }
        out.push_str(";");
        return out;
    }
}

#[derive(Debug, Default)]
pub struct ExpressionStatement {
    pub token: Token,
    pub expression: Option<ExpressionNode>,
}

impl Node for ExpressionStatement {
    fn token_literal(&self) -> String {
        return self.token.literal.clone();
    }
    fn print_string(&self) -> String {
        if let Some(expression) = &self.expression {
            return expression.print_string();
        }
        String::from("")
    }
}

#[derive(Debug)]
pub struct IntegerLiteral {
    pub token: Token,
    pub value: i64,
}

impl Node for IntegerLiteral {
    fn token_literal(&self) -> String {
        return self.token.literal.clone();
    }
    fn print_string(&self) -> String {
        return self.token_literal();
    }
}

#[derive(Debug, Default)]
pub struct PrefixExpression {
    pub token: Token,
    pub operator: String,
    pub right: Box<ExpressionNode>,
}

impl Node for PrefixExpression {
    fn token_literal(&self) -> String {
        return self.token.literal.clone();
    }
    fn print_string(&self) -> String {
        let mut out = String::new();
        out.push_str("(");
        out.push_str(self.operator.as_str());
        out.push_str(self.right.print_string().as_str());
        out.push_str(")");
        out
    }
}

#[derive(Debug, Default)]
pub struct InfixExpression {
    pub token: Token,
    pub left: Box<ExpressionNode>,
    pub operator: String,
    pub right: Box<ExpressionNode>,
}

impl Node for InfixExpression {
    fn token_literal(&self) -> String {
        return self.token.literal.clone();
    }
    fn print_string(&self) -> String {
        let mut out = String::new();
        out.push_str("(");
        out.push_str(self.left.print_string().as_str());
        out.push_str(format!(" {} ", self.operator).as_str());
        out.push_str(self.right.print_string().as_str());
        out.push_str(")");
        out
    }
}

#[derive(Debug)]
pub struct Boolean {
    pub token: Token,
    pub value: bool,
}

impl Node for Boolean {
    fn token_literal(&self) -> String {
        return self.token.literal.clone();
    }
    fn print_string(&self) -> String {
        return self.token_literal();
    }
}
#[derive(Debug, Default)]
pub struct IfExpression {
    pub token: Token,
    pub condition: Box<ExpressionNode>,
    pub consequence: BlockStatement,
    pub alternative: Option<BlockStatement>,
}

impl Node for IfExpression {
    fn token_literal(&self) -> String {
        return self.token.literal.clone();
    }
    fn print_string(&self) -> String {
        let mut out = String::new();
        out.push_str("if");
        out.push_str(self.condition.print_string().as_str());
        out.push_str(" ");
        out.push_str(self.consequence.print_string().as_str());
        if let Some(alt) = &self.alternative {
            out.push_str("else ");
            out.push_str(alt.print_string().as_str());
        }
        out
    }
}

#[derive(Debug, Default)]
pub struct BlockStatement {
    pub token: Token,
    pub statements: Vec<StatementNode>,
}

impl Node for BlockStatement {
    fn token_literal(&self) -> String {
        return self.token.literal.clone();
    }
    fn print_string(&self) -> String {
        let mut out = String::new();
        for statement in &self.statements {
            out.push_str(statement.print_string().as_str());
        }
        out
    }
}

#[derive(Debug)]
pub struct FunctionLiteral {
    pub token: Token,
    pub parameters: Vec<Identifier>,
    pub body: BlockStatement,
}

impl Node for FunctionLiteral {
    fn token_literal(&self) -> String {
        return self.token.literal.clone();
    }
    fn print_string(&self) -> String {
        let mut out = String::new();
        out.push_str(self.token_literal().as_str());
        out.push_str("(");
        for (i, param) in self.parameters.iter().enumerate() {
            out.push_str(param.print_string().as_str());
            if i != self.parameters.len() - 1 {
                out.push_str(", ");
            }
        }
        out.push_str(") ");
        out.push_str(self.body.print_string().as_str());
        out
    }
}

#[derive(Debug, Default)]
pub struct CallExpression {
    pub token: Token,
    pub function: Box<ExpressionNode>,
    pub arguments: Vec<ExpressionNode>,
}

impl Node for CallExpression {
    fn token_literal(&self) -> String {
        return self.token.literal.clone();
    }
    fn print_string(&self) -> String {
        let mut out = String::new();
        out.push_str(self.function.print_string().as_str());
        out.push_str("(");
        for (i, arg) in self.arguments.iter().enumerate() {
            out.push_str(arg.print_string().as_str());
            if i != self.arguments.len() - 1 {
                out.push_str(", ");
            }
        }
        out.push_str(")");
        out
    }
}

#[cfg(test)]
mod test {
    use super::{ExpressionNode, Identifier, LetStatement, Node, Program, StatementNode};
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
            program.print_string(),
            "let myVar = anotherVar;",
            "program.print_string() wrong. got={}",
            program.print_string()
        );
    }
}
