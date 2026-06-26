use crate::object::{Object, NULL};

pub struct Builtins;

impl Builtins {
    pub fn all_builtins(&self) -> Vec<(String, Object)> {
        vec![
            (String::from("len"), Object::Builtin(b_len)),
            (String::from("first"), Object::Builtin(b_first)),
            (String::from("last"), Object::Builtin(b_last)),
            (String::from("rest"), Object::Builtin(b_rest)),
            (String::from("push"), Object::Builtin(b_push)),
            (String::from("log"), Object::Builtin(b_log)),
        ]
    }
}

fn b_len(args: Vec<Object>) -> Object {
    if args.len() != 1 {
        return Object::Error(format!(
            "wrong number of arguments. got={}, want=1",
            args.len()
        ));
    }

    match &args[0] {
        Object::StringObj(string_lit) => Object::Integer(string_lit.len() as i64),
        Object::Array(arr) => Object::Integer(arr.len() as i64),
        other => Object::Error(format!(
            "argument to `len` not supported, got {}",
            other.object_type()
        )),
    }
}

fn b_first(args: Vec<Object>) -> Object {
    if args.len() != 1 {
        return Object::Error(format!(
            "wrong number of arguments. got={}, want=1",
            args.len()
        ));
    }

    match &args[0] {
        Object::Array(arr) => {
            if !arr.is_empty() {
                arr[0].clone()
            } else {
                NULL
            }
        }
        other => Object::Error(format!(
            "argument to `first` not supported, got {}",
            other.object_type()
        )),
    }
}

fn b_last(args: Vec<Object>) -> Object {
    if args.len() != 1 {
        return Object::Error(format!(
            "wrong number of arguments. got={}, want=1",
            args.len()
        ));
    }

    match &args[0] {
        Object::Array(arr) => {
            if !arr.is_empty() {
                arr[arr.len() - 1].clone()
            } else {
                NULL
            }
        }
        other => Object::Error(format!(
            "argument to `last` not supported, got {}",
            other.object_type()
        )),
    }
}

fn b_rest(args: Vec<Object>) -> Object {
    if args.len() != 1 {
        return Object::Error(format!(
            "wrong number of arguments. got={}, want=1",
            args.len()
        ));
    }

    match &args[0] {
        Object::Array(arr) => {
            if !arr.is_empty() {
                Object::Array(arr[1..].to_vec())
            } else {
                NULL
            }
        }
        other => Object::Error(format!(
            "argument to `rest` not supported, got {}",
            other.object_type()
        )),
    }
}

fn b_push(args: Vec<Object>) -> Object {
    if args.len() != 2 {
        return Object::Error(format!(
            "wrong number of arguments. got={}, want=2",
            args.len()
        ));
    }

    match &args[0] {
        Object::Array(arr) => {
            if !arr.is_empty() {
                let mut new_elements = arr.clone();
                new_elements.push(args[1].clone());
                return Object::Array(new_elements);
            } else {
                NULL
            }
        }
        other => Object::Error(format!(
            "argument to `rest` not supported, got {}",
            other.object_type()
        )),
    }
}

fn b_log(args: Vec<Object>) -> Object {
    for arg in args {
        println!("{}", arg);
    }
    NULL
}
