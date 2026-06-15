use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

use crate::ast::{BlockStatement, Identifier, Node};

#[derive(Debug, Clone)]
pub enum Object {
    Integer(i64),
    Boolean(bool),
    ReturnValue(Box<Object>),
    Error(String),
    Func(Function),
    Null,
}

impl Object {
    pub fn object_type(&self) -> String {
        match self {
            Object::Integer(_) => String::from("INTEGER"),
            Object::Boolean(_) => String::from("BOOLEAN"),
            Object::ReturnValue(_) => String::from("RETURN_VALUE"),
            Object::Error(_) => String::from("ERROR"),
            Object::Func(_) => String::from("FUNCTION"),
            Object::Null => String::from("NULL"),
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Object::Integer(value) => write!(f, "{}", value),
            Object::Boolean(value) => write!(f, "{}", value),
            Object::ReturnValue(ret_value) => write!(f, "{}", ret_value),
            Object::Error(message) => write!(f, "ERROR: {}", message),
            Object::Func(function) => {
                let mut out = String::from("");
                let mut params = vec![];

                for p in &function.parameters {
                    params.push(p.print_string());
                }
                out.push_str("fn");
                out.push_str("(");
                out.push_str(&params.join(", ").as_str());
                out.push_str(") {\n");
                out.push_str(&function.body.print_string().as_str());
                out.push_str("\n}");

                write!(f, "{}", out)
            }
            Object::Null => write!(f, "null"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Environment {
    pub store: HashMap<String, Object>,
    pub outer: Option<Box<Environment>>,
}

impl Environment {
    pub fn new_environment() -> Environment {
        Environment {
            store: HashMap::new(),
            outer: None,
        }
    }

    pub fn new_enclosed_environment(outer: Box<Environment>) -> Environment {
        let env_map = HashMap::new();
        Environment {
            store: env_map,
            outer: Some(outer),
        }
    }

    pub fn get(&self, name: String) -> Option<Object> {
        match self.store.get(&name) {
            Some(obj) => Some(obj.clone()),
            None => match &self.outer {
                Some(env) => env.get(name),
                None => None,
            },
        }
    }

    pub fn set(&mut self, name: String, value: Object) -> Option<Object> {
        self.store.insert(name.clone(), value);
        return self.get(name);
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub parameters: Vec<Identifier>,
    pub body: BlockStatement,
    pub env: Environment,
}
