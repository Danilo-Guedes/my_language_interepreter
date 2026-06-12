use std::{
    collections::HashMap,
    fmt::{self, Display, Formatter},
};

#[derive(Debug, Clone)]
pub enum Object {
    Integer(i64),
    Boolean(bool),
    ReturnValue(Box<Object>),
    Error(String),
    Null,
}

impl Object {
    pub fn object_type(&self) -> String {
        match self {
            Object::Integer(_) => String::from("INTEGER"),
            Object::Boolean(_) => String::from("BOOLEAN"),
            Object::ReturnValue(_) => String::from("RETURN_VALUE"),
            Object::Error(_) => String::from("ERROR"),
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
            Object::Null => write!(f, "null"),
        }
    }
}

#[derive(Debug)]
pub struct Environment {
    pub store: HashMap<String, Object>,
}

impl Environment {
    pub fn new_environment() -> Environment {
        Environment {
            store: HashMap::new(),
        }
    }

    pub fn get(&self, name: String) -> Option<Object> {
        match self.store.get(&name) {
            Some(obj) => Some(obj.clone()),
            None => None,
        }
    }

    pub fn set(&mut self, name: String, value: Object) -> Option<Object> {
        self.store.insert(name.clone(), value);
        return self.get(name);
    }
}
