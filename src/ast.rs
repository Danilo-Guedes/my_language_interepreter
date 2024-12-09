use crate::token::Token;

pub trait Node {
    fn token_literal(&self) -> String;
    fn print_string(&self) -> String;
}

#[derive(Debug)]
pub enum StatementNode {
    Let(LetStatement),
    Return(ReturnStatement),
}

impl Node for StatementNode {
    fn token_literal(&self) -> String {
        return match self {
            Self::Let(let_stmt) => let_stmt.token_literal(),
            Self::Return(return_stmt) => return_stmt.token_literal(),
        };
    }

    fn print_string(&self) -> String {
        return match self {
            Self::Let(let_stmt) => let_stmt.print_string(),
            Self::Return(return_stmt) => return_stmt.print_string(),
        };
    }
}

#[derive(Debug)]

pub enum ExpressionNode {
    IdentifierNode(Identifier),
}

impl Node for ExpressionNode {
    fn token_literal(&self) -> String {
        return match self {
            Self::IdentifierNode(identifirer) => identifirer.token_literal(),
        };
    }

    fn print_string(&self) -> String {
        return match self {
            Self::IdentifierNode(identifier) => identifier.print_string(),
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
