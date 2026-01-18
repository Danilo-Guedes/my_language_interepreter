use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum Object {
    Integer(i64),
    Boolean(bool),
    ReturnValue(Box<Object>),
    Null,
}

impl Object {
    pub fn object_type(&self) -> String {
        match self {
            Object::Integer(_) => String::from("INTEGER"),
            Object::Boolean(_) => String::from("BOOLEAN"),
            Object::Null => String::from("NULL"),
            Object::ReturnValue(_) => String::from("RETURN_VALUE"),
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Object::Integer(value) => write!(f, "{}", value),
            Object::Boolean(value) => write!(f, "{}", value),
            Object::ReturnValue(ret_value) => write!(f, "{}", ret_value),
            Object::Null => write!(f, "null"),
        }
    }
}
