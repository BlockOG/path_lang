use std::{
    cmp::Ordering,
    collections::{HashMap, VecDeque},
    fmt::Display,
};

use num_bigint::{BigUint, ToBigUint};

use crate::{instruction::Instruction, Value};
use anyhow::Result;
use thiserror::Error;

#[derive(Default, Clone, PartialEq)]
pub(crate) enum BuiltInFunction {
    #[default]
    None,
    Print,
    PrintLn,
    // Read,
    ReadLn,
    ToStr,
    ToInt,
    Trim,
}

#[derive(Debug, Error)]
pub(crate) enum FunctionCallError {
    #[error("Invalid number of arguments: expected {expected}, got {got}")]
    InvalidNumberOfArguments { expected: usize, got: usize },
}

#[derive(Default, Clone)]
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
            if self.built_in != BuiltInFunction::None {
                "built-in"
            } else {
                "user-defined"
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
                if optionals.contains_key(&0.to_biguint().unwrap()) {
                    if let Value::String(s) = optionals.get(&0.to_biguint().unwrap()).unwrap() {
                        sep = s;
                    }
                }

                for (i, arg) in args.into_iter().enumerate() {
                    if i > 0 {
                        print!("{sep}");
                    }
                    match arg {
                        Value::Integer(i) => print!("{i}"),
                        Value::String(s) => print!("{s}"),
                        Value::Function(f) => print!("{f}"),
                    }
                }

                if self.built_in == BuiltInFunction::PrintLn {
                    print!("\n");
                }
            }
            BuiltInFunction::ReadLn => {
                let mut input = String::new();
                std::io::stdin().read_line(&mut input).unwrap();
                return Ok(Some(Value::String(input)));
            }
            BuiltInFunction::ToStr => {
                let arg = args.into_iter().next().unwrap();
                return Ok(Some(Value::String(match arg {
                    Value::Integer(i) => i.to_string(),
                    Value::String(s) => s,
                    Value::Function(f) => f.to_string(),
                })));
            }
            BuiltInFunction::ToInt => {
                let arg = args.into_iter().next().unwrap();
                return Ok(Some(Value::Integer(match arg {
                    Value::Integer(i) => i,
                    Value::String(s) => s.parse().unwrap(),
                    Value::Function(_) => {
                        panic!("Cannot convert function to integer")
                    }
                })));
            }
            BuiltInFunction::Trim => {
                let arg = args.into_iter().next().unwrap();
                return Ok(Some(Value::String(match arg {
                    Value::String(s) => s.trim().to_owned(),
                    Value::Integer(_) | Value::Function(_) => {
                        panic!("Cannot trim integer or function")
                    }
                })));
            }
        }
        Ok(None)
    }
}
