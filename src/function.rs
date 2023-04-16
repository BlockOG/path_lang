use std::{
    cmp::Ordering,
    collections::{HashMap, VecDeque},
    fmt::Display,
    io::stdin,
};

use num_bigint::BigUint;
use num_traits::Zero;

use crate::{instruction::Instruction, Value};
use anyhow::Result;
use thiserror::Error;

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) enum BuiltInFunction {
    #[default]
    None,
    Print,
    PrintLn,
    // Read,
    ReadLn,
    ToBool,
    ToStr,
    ToInt,
    Trim,
    Len,
    Push,
    Pop,
}

#[derive(Debug, Error)]
pub(crate) enum FunctionCallError {
    #[error("Invalid number of arguments: expected {expected}, got {got}")]
    InvalidNumberOfArguments { expected: usize, got: usize },
}

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct Function {
    varargs: bool,
    arity: usize,

    built_in: BuiltInFunction,
    instructions: Vec<Instruction>,
}

impl Display for Function {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<function {} arity={} {}>",
            if self.varargs { "varargs" } else { "constant" },
            self.arity,
            if let BuiltInFunction::None = self.built_in {
                "user-defined"
            } else {
                "built-in"
            }
        )
    }
}

impl Function {
    pub(crate) fn new(varargs: bool, arity: usize, instructions: Vec<Instruction>) -> Function {
        Function {
            varargs,
            arity,
            built_in: BuiltInFunction::None,
            instructions,
        }
    }

    pub(crate) fn new_built_in(varargs: bool, arity: usize, built_in: BuiltInFunction) -> Function {
        Function {
            varargs,
            arity,
            built_in,
            instructions: vec![],
        }
    }

    pub(crate) fn call(
        &self,
        args: VecDeque<Value>,
        optionals: HashMap<BigUint, Value>,
    ) -> Result<Option<Value>> {
        if match args.len().cmp(&self.arity) {
            Ordering::Less => true,
            Ordering::Equal => false,
            Ordering::Greater => !self.varargs,
        } {
            return Err(FunctionCallError::InvalidNumberOfArguments {
                expected: self.arity,
                got: args.len(),
            }
            .into());
        }

        match self.built_in {
            BuiltInFunction::None => {}
            BuiltInFunction::Print | BuiltInFunction::PrintLn => {
                let mut sep = " ";
                if optionals.contains_key(&BigUint::zero()) {
                    if let Value::String(s) = optionals.get(&BigUint::zero()).unwrap() {
                        sep = s;
                    }
                }

                for (i, arg) in args.into_iter().enumerate() {
                    if i > 0 {
                        print!("{sep}");
                    }
                    print!("{arg}");
                }

                if let BuiltInFunction::PrintLn = self.built_in {
                    print!("\n");
                }
            }
            BuiltInFunction::ReadLn => {
                let mut input = String::new();
                stdin().read_line(&mut input).unwrap();
                return Ok(Some(Value::String(input)));
            }
            BuiltInFunction::ToBool => {
                return Ok(Some(Value::Boolean(
                    args.into_iter().next().unwrap().into(),
                )));
            }
            BuiltInFunction::ToStr => {
                return Ok(Some(Value::String(
                    args.into_iter().next().unwrap().to_string(),
                )));
            }
            BuiltInFunction::ToInt => {
                return Ok(Some(Value::Integer(
                    args.into_iter().next().unwrap().into(),
                )));
            }
            BuiltInFunction::Trim => {
                return Ok(Some(Value::String(
                    match args.into_iter().next().unwrap() {
                        Value::String(s) => s.trim().to_owned(),
                        _ => {
                            panic!("Can only trim strings")
                        }
                    },
                )));
            }
            BuiltInFunction::Len => {
                return Ok(Some(Value::Integer(
                    match args.into_iter().next().unwrap() {
                        Value::String(s) => s.len(),
                        Value::Array(a) => a.len(),
                        _ => {
                            panic!("Can only get length of strings and arrays")
                        }
                    }
                    .into(),
                )));
            }
            BuiltInFunction::Push => {
                let mut args = args.into_iter();
                match args.next().unwrap().into() {
                    Value::Array(mut array) => {
                        array.push(args.next().unwrap().into());
                        return Ok(Some(Value::Array(array)));
                    }
                    Value::String(mut string) => {
                        match args.next().unwrap().into() {
                            Value::String(s) => string.push_str(&s),
                            Value::Integer(i) => string.push(TryInto::<u8>::try_into(i)? as char),
                            _ => {
                                panic!("Can only push strings or characters to strings")
                            }
                        }
                        return Ok(Some(Value::String(string)));
                    }
                    _ => {
                        panic!("Can only push to arrays or strings")
                    }
                }
            }
            BuiltInFunction::Pop => {
                let mut args = args.into_iter();
                match args.next().unwrap().into() {
                    Value::Array(mut array) => {
                        return Ok(Some(Value::Array(vec![
                            array.pop().unwrap().into(),
                            Value::Array(array),
                        ])));
                    }
                    Value::String(mut string) => {
                        return Ok(Some(Value::Array(vec![
                            Value::Integer((string.pop().unwrap() as u8).into()),
                            Value::String(string),
                        ])));
                    }
                    _ => {
                        panic!("Can only pop from arrays or strings")
                    }
                }
            }
        }
        Ok(None)
    }
}
