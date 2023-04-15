use std::collections::{HashMap, VecDeque};

use num_bigint::{BigUint, ToBigUint};

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

    pub(crate) fn call(
        &self,
        args: VecDeque<Value>,
        optionals: HashMap<BigUint, Value>,
    ) -> Option<Value> {
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

                if self.built_in == BuiltInFunction::PrintLn {
                    print!("\n");
                }
            }
        }
        None
    }
}
