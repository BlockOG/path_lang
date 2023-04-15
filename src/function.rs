use std::collections::VecDeque;

use crate::{instruction::Instruction, Value};

#[derive(Default, Clone, PartialEq)]
pub(crate) enum BuiltInFunction {
    #[default]
    None,
    Print,
    PrintLn,
}

#[derive(Default, Clone)]
pub(crate) struct Function {
    varargs: bool,
    arity: usize,

    built_in: BuiltInFunction,
    instructions: Vec<Instruction>,
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

    pub(crate) fn call(&self, args: VecDeque<Value>) -> Option<Value> {
        match self.built_in {
            BuiltInFunction::None => {}
            BuiltInFunction::Print => {
                for arg in args {
                    match arg {
                        Value::Integer(i) => print!("{}", i),
                        Value::String(s) => print!("{}", s),
                        Value::Function(f) => {
                            print!(
                                "<function {} arity={} {}>",
                                if f.varargs { "varargs" } else { "constant" },
                                f.arity,
                                if f.built_in != BuiltInFunction::None {
                                    "built-in"
                                } else {
                                    "user-defined"
                                }
                            )
                        }
                    }
                }
            }
            BuiltInFunction::PrintLn => {
                for arg in args {
                    match arg {
                        Value::Integer(i) => println!("{}", i),
                        Value::String(s) => println!("{}", s),
                        Value::Function(f) => {
                            println!(
                                "<function {} arity={} {}>",
                                if f.varargs { "varargs" } else { "constant" },
                                f.arity,
                                if f.built_in != BuiltInFunction::None {
                                    "built-in"
                                } else {
                                    "user-defined"
                                }
                            )
                        }
                    }
                }
            }
        }
        None
    }
}
