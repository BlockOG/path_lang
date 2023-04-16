mod function;
mod instruction;
mod memory;

use std::{
    cmp::Ordering,
    collections::{HashMap, VecDeque},
    env,
};

use anyhow::Result;
use function::Function;
use instruction::{Instruction, ParseInstruction, ParsedInstruction};
use memory::Memory;
use num_bigint::{BigInt, BigUint, ToBigInt, ToBigUint};
use thiserror::Error;

#[derive(Debug, Error)]
enum SyntaxError {
    #[error("Doesn't start at /")]
    NotNoop,
}

#[derive(Debug, Error)]
enum RuntimeError {
    #[error("Stack underflow")]
    StackUnderflow,
    #[error("Not a function")]
    NotAFunc,
    #[error("Invalid instruction")]
    InvalidInstruction,
}

#[derive(Clone)]
enum StackValue {
    Value(Value),
    Optional(BigUint, Value),
    CallStart,
}

#[derive(Clone)]
enum Value {
    Integer(BigInt),
    // Float(BigFloat),
    String(String),
    Function(Function),
}

fn parse(path: String) -> Result<Vec<Instruction>> {
    let mut chars = path.chars().peekable();
    if chars.next().expect("How did you even run this?") != '/' {
        return Err(SyntaxError::NotNoop.into());
    }

    let mut instructions = Vec::new();

    loop {
        match chars.parse_instruction() {
            ParsedInstruction::Instruction(instruction) => {
                instructions.push(instruction);
            }
            ParsedInstruction::Err(e) => return Err(e.into()),
            ParsedInstruction::CodePartStopped => break,
        }
    }

    Ok(instructions)
}

fn run(instructions: Vec<Instruction>) -> Result<()> {
    let mut stack: Vec<StackValue> = Vec::new();
    let mut memory = Memory::new();

    let mut ptr = 0;
    let mut cmp = None;
    while ptr < instructions.len() {
        let mut jumped = false;
        let instruction = &instructions[ptr];

        match instruction.len() {
            0 => {}
            1 => {
                if !instruction[0] {
                    // duplicate top of stack
                    let value = stack.last().ok_or(RuntimeError::StackUnderflow)?.clone();
                    stack.push(value);
                } else {
                    // pop top of stack
                    stack.pop().ok_or(RuntimeError::StackUnderflow)?;
                }
            }
            2 => {
                if !instruction[0] {
                    if !instruction[1] {
                        // push integer
                        let int: BigUint = match instructions.get(ptr + 2) {
                            Some(instruction) => instruction,
                            None => return Err(RuntimeError::InvalidInstruction.into()),
                        }
                        .into();
                        if match instructions.get(ptr + 1) {
                            Some(instruction) => instruction,
                            None => return Err(RuntimeError::InvalidInstruction.into()),
                        }
                        .into()
                        {
                            stack.push(StackValue::Value(Value::Integer(int.to_bigint().unwrap())));
                        } else {
                            stack
                                .push(StackValue::Value(Value::Integer(-int.to_bigint().unwrap())));
                        }

                        ptr += 2;
                    } else {
                        // pop to variable
                        ptr += 1;
                        let index: BigUint = match instructions.get(ptr) {
                            Some(instruction) => instruction,
                            None => return Err(RuntimeError::InvalidInstruction.into()),
                        }
                        .into();

                        memory.set(
                            index,
                            match stack.pop().ok_or(RuntimeError::StackUnderflow)? {
                                StackValue::Value(value) => value,
                                StackValue::CallStart | StackValue::Optional(_, _) => {
                                    return Err(RuntimeError::InvalidInstruction.into())
                                }
                            },
                        );
                    }
                } else {
                    if !instruction[1] {
                        // push variable
                        ptr += 1;
                        let index: BigUint = match instructions.get(ptr) {
                            Some(instruction) => instruction,
                            None => return Err(RuntimeError::InvalidInstruction.into()),
                        }
                        .into();

                        stack.push(StackValue::Value(
                            memory
                                .get(index)
                                .ok_or(RuntimeError::InvalidInstruction)?
                                .clone(),
                        ));
                    } else {
                        // push string
                        ptr += 1;
                        let mut length: BigUint = match instructions.get(ptr) {
                            Some(instruction) => instruction,
                            None => return Err(RuntimeError::InvalidInstruction.into()),
                        }
                        .into();

                        let mut string = String::new();
                        while length > 0.to_biguint().unwrap() {
                            length -= 1.to_biguint().unwrap();
                            ptr += 1;
                            string.push(
                                Into::<Result<u8>>::into(match instructions.get(ptr) {
                                    Some(instruction) => instruction,
                                    None => return Err(RuntimeError::InvalidInstruction.into()),
                                })?
                                .into(),
                            );
                        }

                        stack.push(StackValue::Value(Value::String(string)));
                    }
                }
            }
            3 => {
                if !instruction[0] {
                    if !instruction[1] {
                        if !instruction[2] {
                            // push float
                            todo!("Floats");
                        } else {
                            // function call start
                            stack.push(StackValue::CallStart);
                        }
                    } else {
                        if !instruction[2] {
                            // call function
                            ptr += 1;
                            let function = memory.get(
                                match instructions.get(ptr) {
                                    Some(instruction) => instruction,
                                    None => return Err(RuntimeError::InvalidInstruction.into()),
                                }
                                .into(),
                            );
                            if let Some(Value::Function(function)) = function {
                                let mut args = VecDeque::new();
                                let mut optionals = HashMap::new();
                                loop {
                                    match stack.pop() {
                                        Some(StackValue::Value(value)) => args.push_front(value),
                                        Some(StackValue::Optional(index, value)) => {
                                            optionals.insert(index, value);
                                        }
                                        Some(StackValue::CallStart) => break,
                                        None => return Err(RuntimeError::StackUnderflow.into()),
                                    }
                                }
                                if let Some(value) = function.call(args, optionals)? {
                                    stack.push(StackValue::Value(value));
                                }
                            } else {
                                return Err(RuntimeError::NotAFunc.into());
                            }
                        } else {
                            // make optional argument
                            let value = stack.pop().ok_or(RuntimeError::StackUnderflow)?;
                            ptr += 1;
                            let index: BigUint = match instructions.get(ptr) {
                                Some(instruction) => instruction,
                                None => return Err(RuntimeError::InvalidInstruction.into()),
                            }
                            .into();

                            stack.push(StackValue::Optional(
                                index,
                                match value {
                                    StackValue::Value(value) => value,
                                    StackValue::CallStart | StackValue::Optional(_, _) => {
                                        return Err(RuntimeError::InvalidInstruction.into())
                                    }
                                },
                            ));
                        }
                    }
                } else {
                    match (instruction[1], instruction[2], cmp) {
                        (false, false, _) // jump to instruction
                        | (false, true, Some(Ordering::Less)) // jump to instruction if less
                        | (true, false, Some(Ordering::Equal)) // jump to instruction if equal
                        | (true, true, Some(Ordering::Greater)) // jump to instruction if greater
                        => {
                            ptr = Into::<usize>::into(match instructions.get(ptr + 1) {
                                Some(instruction) => instruction,
                                None => return Err(RuntimeError::InvalidInstruction.into()),
                            });
                            jumped = true;
                        }
                        _ => ptr += 1,
                    }
                }
            }
            4 => {
                if !instruction[0] {
                    if !instruction[1] {
                        if !instruction[2] {
                            if !instruction[3] {
                                // compare top two stack items
                                let a = match stack.last().ok_or(RuntimeError::StackUnderflow)? {
                                    StackValue::Value(Value::Integer(int)) => int,
                                    _ => return Err(RuntimeError::InvalidInstruction.into()),
                                };
                                let b = match stack
                                    .get(stack.len() - 2)
                                    .ok_or(RuntimeError::StackUnderflow)?
                                {
                                    StackValue::Value(Value::Integer(int)) => int,
                                    _ => return Err(RuntimeError::InvalidInstruction.into()),
                                };
                                cmp = Some(a.cmp(b));
                            } else {
                                // remove variable
                                let index: BigUint = match instructions.get(ptr + 1) {
                                    Some(instruction) => instruction,
                                    None => return Err(RuntimeError::InvalidInstruction.into()),
                                }
                                .into();

                                memory.remove(index);
                            }
                        } else {
                            return Err(RuntimeError::InvalidInstruction.into());
                        }
                    } else {
                        return Err(RuntimeError::InvalidInstruction.into());
                    }
                } else {
                    return Err(RuntimeError::InvalidInstruction.into());
                }
            }
            _ => {
                return Err(RuntimeError::InvalidInstruction.into());
            }
        }

        if !jumped {
            ptr += 1;
        }
    }

    Ok(())
}

fn main() -> Result<()> {
    match env::consts::OS {
        "windows" => {
            let mut args = env::args();
            args.next().expect("How did you even run this?");
            run(parse(args.next().expect(
                "Unfortunately for you Windows people, you need to pass a Linux path as the second argument",
            ))?)
        }
        _ => run(parse(
            env::args().next().expect("How did you even run this?"),
        )?),
    }
}
