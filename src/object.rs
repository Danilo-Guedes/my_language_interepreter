use std::fmt::{self, Display, Formatter};

#[derive(Debug)]
pub enum Object {
    Integer(i64),
}

impl Object {
    pub fn object_type(&self) -> String {
        match self {
            Object::Integer(_) => String::from("INTEGER"),
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            Object::Integer(value) => write!(f, "{}", value),
        }
    }
}
