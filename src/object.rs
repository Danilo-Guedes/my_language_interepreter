use std::{
    cell::RefCell,
    collections::HashMap,
    fmt::{self, Display, Formatter},
    hash::{DefaultHasher, Hash, Hasher},
    rc::Rc,
};

use crate::{
    ast::{BlockStatement, Identifier, Node},
    builtins::Builtins,
};

pub type BuiltinFunction = fn(Vec<Object>) -> Object;
pub type Env = Rc<RefCell<Environment>>;

pub const TRUE: Object = Object::Boolean(true);
pub const FALSE: Object = Object::Boolean(false);
pub const NULL: Object = Object::Null;

#[derive(Debug, Clone)]
pub enum Object {
    Integer(i64),
    Boolean(bool),
    ReturnValue(Box<Object>),
    Error(String),
    Func(Function),
    StringObj(String),
    Builtin(BuiltinFunction),
    Array(Vec<Object>),
    HashObj(HashStruct),
    Null,
}

impl Object {
    pub fn object_type(&self) -> String {
        match self {
            Self::Integer(_) => String::from("INTEGER"),
            Self::Boolean(_) => String::from("BOOLEAN"),
            Self::ReturnValue(_) => String::from("RETURN_VALUE"),
            Self::Error(_) => String::from("ERROR"),
            Self::Func(_) => String::from("FUNCTION"),
            Self::StringObj(_) => String::from("STRING"),
            Self::Builtin(_) => String::from("BUILTIN"),
            Self::Array(_) => String::from("ARRAY"),
            Self::HashObj(_) => String::from("HASH"),
            Self::Null => String::from("NULL"),
        }
    }
}

impl Display for Object {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Integer(value) => write!(f, "{}", value),
            Self::Boolean(value) => write!(f, "{}", value),
            Self::ReturnValue(ret_value) => write!(f, "{}", ret_value),
            Self::Error(message) => write!(f, "ERROR: {}", message),
            Self::Func(function) => {
                let mut out = String::from("");
                let mut params = vec![];

                for p in &function.parameters {
                    params.push(p.print_string());
                }
                out.push_str("fn");
                out.push('(');
                out.push_str(params.join(", ").as_str());
                out.push_str(") {\n");
                out.push_str(function.body.print_string().as_str());
                out.push_str("\n}");

                write!(f, "{}", out)
            }
            Self::StringObj(str) => write!(f, "{}", str),
            Self::Array(elements) => {
                let mut out = String::from("[");
                let mut elems = vec![];
                for el in elements {
                    elems.push(format!("{}", el));
                }
                out.push_str(&elems.join(", "));
                out.push(']');
                write!(f, "{}", out)
            }
            Self::Builtin(_) => write!(f, "builtin function"),
            Self::HashObj(hash) => {
                let mut out = String::from("{");
                let mut pairs = vec![];
                for pair in hash.pairs.values() {
                    pairs.push(format!("{}: {}", pair.key, pair.value));
                }
                out.push_str(&pairs.join(", "));
                out.push('}');
                write!(f, "{}", out)
            }
            Self::Null => write!(f, "null"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Environment {
    pub store: HashMap<String, Object>,
    pub outer: Option<Env>,
}

impl Environment {
    pub fn new_environment() -> Env {
        let mut env_map = HashMap::new();
        Self::init_builtins(&mut env_map);

        Rc::new(RefCell::new(Environment {
            store: env_map,
            outer: None,
        }))
    }

    fn init_builtins(hashmap: &mut HashMap<String, Object>) {
        let builtins_functions = Builtins;
        let builtins = builtins_functions.all_builtins();
        for (name, builtin) in builtins {
            hashmap.insert(name, builtin);
        }
    }

    pub fn new_enclosed_environment(outer: Env) -> Env {
        let env_map = HashMap::new();
        Rc::new(RefCell::new(Environment {
            store: env_map,
            outer: Some(outer),
        }))
    }

    pub fn get(&self, name: String) -> Option<Object> {
        match self.store.get(&name) {
            Some(obj) => Some(obj.clone()),
            None => match &self.outer {
                Some(outer) => outer.borrow().get(name),
                None => None,
            },
        }
    }

    pub fn set(&mut self, name: String, value: Object) {
        self.store.insert(name.clone(), value);
    }
}

#[derive(Debug, Clone)]
pub struct Function {
    pub parameters: Vec<Identifier>,
    pub body: BlockStatement,
    pub env: Env,
}

#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub struct HashKey {
    pub object_type: String,
    pub value: i64,
}

pub trait Hashable {
    fn hash_key(&self) -> Result<HashKey, String>;
}

impl Hashable for Object {
    fn hash_key(&self) -> Result<HashKey, String> {
        match self {
            Self::Boolean(bool) => Ok(HashKey {
                object_type: self.object_type(),
                value: if *bool { 1 } else { 0 },
            }),
            Self::Integer(int) => Ok(HashKey {
                object_type: self.object_type(),
                value: *int,
            }),
            Self::StringObj(string) => {
                let mut hasher = DefaultHasher::new();
                string.hash(&mut hasher);
                Ok(HashKey {
                    object_type: self.object_type(),
                    value: hasher.finish() as i64,
                })
            }
            _ => Err(format!("unusable as hash key: {}", self.object_type())),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HashPair {
    pub key: Object,
    pub value: Object,
}

#[derive(Debug, Clone)]
pub struct HashStruct {
    pub pairs: HashMap<HashKey, HashPair>,
}

#[cfg(test)]
mod test {

    use super::{Hashable, Object};

    #[test]
    fn test_string_hash_key() {
        let hello1 = Object::StringObj("Hello World".to_string());
        let hello2 = Object::StringObj("Hello World".to_string());

        let some_other = Object::StringObj("Some Other".to_string());

        assert_eq!(
            hello1.hash_key(),
            hello2.hash_key(),
            "strings with same content have different hash keys"
        );

        assert_ne!(
            hello1.hash_key(),
            some_other.hash_key(),
            "strings with different content have same hash keys"
        );
    }
}
