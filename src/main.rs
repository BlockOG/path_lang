mod function;
mod instruction;
mod memory;
mod stack_value;
mod value;

use std::{
    cmp::Ordering,
    collections::{HashMap, VecDeque},
    env,
};

use anyhow::Result;

use instruction::{Instruction, ParseInstruction, ParsedInstruction};
use memory::Memory;
use num_bigint::{BigInt, BigUint};
use num_traits::{One, Pow, Zero};
use stack_value::StackValue;
use thiserror::Error;
use value::Value;

#[derive(Debug, Error)]
enum SyntaxError {
    #[error("Doesn't start at /")]
    NotNoop,
}

#[derive(Debug, Error)]
enum RuntimeError {
    #[error("Stack underflow")]
    StackUnderflow,
    #[error("Invalid instruction")]
    InvalidInstruction,
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
                            stack.push(int.into());
                        } else {
                            stack.push((-BigInt::from(int)).into());
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
                                StackValue::Value(value) | StackValue::Argument(value) => value,
                                StackValue::Optional(_, _) => {
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
                        let length = match instructions.get(ptr) {
                            Some(instruction) => instruction,
                            None => return Err(RuntimeError::InvalidInstruction.into()),
                        }
                        .into();

                        let mut string = String::new();
                        for _ in 0..length {
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
                            // call function
                            let mut args = VecDeque::new();
                            let mut optionals = HashMap::new();
                            loop {
                                match stack.pop() {
                                    Some(StackValue::Value(Value::Function(function))) => {
                                        if let Some(value) = function.call(args, optionals)? {
                                            stack.push(StackValue::Value(value));
                                        }
                                        break;
                                    }
                                    Some(StackValue::Value(value))
                                    | Some(StackValue::Argument(value)) => args.push_front(value),
                                    Some(StackValue::Optional(index, value)) => {
                                        optionals.insert(index, value);
                                    }
                                    None => return Err(RuntimeError::StackUnderflow.into()),
                                }
                            }
                        }
                    } else {
                        if !instruction[2] {
                            // make argument
                            let value = stack.pop().ok_or(RuntimeError::StackUnderflow)?;
                            stack.push(StackValue::Argument(match value {
                                StackValue::Value(value) => value,
                                StackValue::Argument(_) | StackValue::Optional(_, _) => {
                                    return Err(RuntimeError::InvalidInstruction.into())
                                }
                            }));
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
                                    StackValue::Value(value) | StackValue::Argument(value) => value,
                                    StackValue::Optional(_, _) => {
                                        return Err(RuntimeError::InvalidInstruction.into())
                                    }
                                },
                            ));
                        }
                    }
                } else {
                    if !instruction[1] && !instruction[2] {
                        // jump to instruction
                        ptr = match instructions.get(ptr + 1) {
                            Some(instruction) => instruction,
                            None => return Err(RuntimeError::InvalidInstruction.into()),
                        }
                        .into();
                        jumped = true;
                    } else {
                        match (
                            instruction[1],
                            instruction[2],
                            stack
                                .pop()
                                .ok_or(RuntimeError::StackUnderflow)?
                                .partial_cmp(&stack.pop().ok_or(RuntimeError::StackUnderflow)?)
                                .ok_or(RuntimeError::InvalidInstruction)?,
                        ) {
                            // push true
                            (false, true, Ordering::Less)
                            | (true, false, Ordering::Equal)
                            | (true, true, Ordering::Greater) => stack.push(true.into()),

                            // push false
                            _ => stack.push(false.into()),
                        }
                    }
                }
            }
            4 => {
                if !instruction[0] {
                    if !instruction[1] {
                        if !instruction[2] {
                            if !instruction[3] {
                                // index array or string
                                let index = stack.pop().ok_or(RuntimeError::StackUnderflow)?;
                                let value = stack.last().ok_or(RuntimeError::StackUnderflow)?;

                                stack.push(match (value, index) {
                                    (
                                        StackValue::Value(Value::Array(array)),
                                        StackValue::Value(Value::Integer(index)),
                                    ) => array
                                        .get(TryInto::<usize>::try_into(index)?)
                                        .ok_or(RuntimeError::InvalidInstruction)?
                                        .clone()
                                        .into(),
                                    (
                                        StackValue::Value(Value::String(string)),
                                        StackValue::Value(Value::Integer(index)),
                                    ) => string
                                        .chars()
                                        .nth(TryInto::<usize>::try_into(index)?)
                                        .ok_or(RuntimeError::InvalidInstruction)?
                                        .into(),
                                    _ => return Err(RuntimeError::InvalidInstruction.into()),
                                });
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
                            // jump to instruction if top of stack is boolean
                            if instruction[3]
                                == match stack.pop().ok_or(RuntimeError::StackUnderflow)? {
                                    StackValue::Value(value) | StackValue::Argument(value) => {
                                        value.into()
                                    }
                                    _ => return Err(RuntimeError::InvalidInstruction.into()),
                                }
                            {
                                ptr = match instructions.get(ptr + 1) {
                                    Some(instruction) => instruction,
                                    None => return Err(RuntimeError::InvalidInstruction.into()),
                                }
                                .into();
                                jumped = true;
                            } else {
                                ptr += 1;
                            }
                        }
                    } else {
                        if !instruction[2] {
                            // push boolean
                            stack.push(StackValue::Value(Value::Boolean(instruction[3])));
                        } else {
                            if !instruction[3] {
                                // push array
                                let length = match instructions.get(ptr + 1) {
                                    Some(instruction) => instruction,
                                    None => return Err(RuntimeError::InvalidInstruction.into()),
                                }
                                .into();

                                let mut array = Vec::new();
                                for _ in 0..length {
                                    array.push(
                                        stack.pop().ok_or(RuntimeError::StackUnderflow)?.into(),
                                    );
                                }
                                stack.push(StackValue::Value(Value::Array(array)));
                            } else {
                                // spread array
                                let array = match stack.pop().ok_or(RuntimeError::StackUnderflow)? {
                                    StackValue::Value(Value::Array(array)) => array,
                                    _ => return Err(RuntimeError::InvalidInstruction.into()),
                                };
                                for value in array.into_iter().rev() {
                                    stack.push(StackValue::Value(value));
                                }
                            }
                        }
                    }
                } else {
                    if instruction[1] && instruction[2] {
                        let v = stack.pop().ok_or(RuntimeError::StackUnderflow)?.into();
                        if !instruction[3] {
                            // -top
                            stack.push(
                                match v {
                                    Value::Boolean(v) => {
                                        if v {
                                            -BigInt::one()
                                        } else {
                                            BigInt::zero()
                                        }
                                    }
                                    Value::Integer(v) => -v,
                                    _ => return Err(RuntimeError::InvalidInstruction.into()),
                                }
                                .into(),
                            );
                        } else {
                            // !top
                            match v {
                                Value::Boolean(v) => {
                                    stack.push(StackValue::Value(Value::Boolean(!v)))
                                }
                                Value::Integer(v) => stack.push((!v).into()),
                                _ => return Err(RuntimeError::InvalidInstruction.into()),
                            }
                        }
                    } else {
                        let v1 = stack.pop().ok_or(RuntimeError::StackUnderflow)?.into();
                        let v2 = stack.pop().ok_or(RuntimeError::StackUnderflow)?.into();
                        let res = match (instruction[1], instruction[2], instruction[3], v1, v2) {
                            // +
                            (false, false, false, Value::Boolean(v1), Value::Boolean(v2)) => {
                                StackValue::Value(Value::Integer(((v1 as u8) + (v2 as u8)).into()))
                            }
                            (false, false, false, Value::Boolean(v1), Value::Integer(v2)) => {
                                StackValue::Value(Value::Integer((v1 as u8) + v2))
                            }
                            (false, false, false, Value::Integer(v1), Value::Boolean(v2)) => {
                                StackValue::Value(Value::Integer(v1 + (v2 as u8)))
                            }
                            (false, false, false, Value::Integer(v1), Value::Integer(v2)) => {
                                StackValue::Value(Value::Integer(v1 + v2))
                            }

                            // -
                            (false, false, true, Value::Boolean(v1), Value::Boolean(v2)) => {
                                StackValue::Value(Value::Integer(((v1 as u8) - (v2 as u8)).into()))
                            }
                            (false, false, true, Value::Boolean(v1), Value::Integer(v2)) => {
                                StackValue::Value(Value::Integer((v1 as u8) - v2))
                            }
                            (false, false, true, Value::Integer(v1), Value::Boolean(v2)) => {
                                StackValue::Value(Value::Integer(v1 - (v2 as u8)))
                            }
                            (false, false, true, Value::Integer(v1), Value::Integer(v2)) => {
                                StackValue::Value(Value::Integer(v1 - v2))
                            }

                            // *
                            (false, true, false, Value::Boolean(v1), Value::Boolean(v2)) => {
                                StackValue::Value(Value::Integer(((v1 as u8) * (v2 as u8)).into()))
                            }
                            (false, true, false, Value::Boolean(v1), Value::Integer(v2)) => {
                                StackValue::Value(Value::Integer((v1 as u8) * v2))
                            }
                            (false, true, false, Value::Integer(v1), Value::Boolean(v2)) => {
                                StackValue::Value(Value::Integer(v1 * (v2 as u8)))
                            }
                            (false, true, false, Value::Integer(v1), Value::Integer(v2)) => {
                                StackValue::Value(Value::Integer(v1 * v2))
                            }

                            // /
                            (false, true, true, Value::Boolean(v1), Value::Boolean(v2)) => {
                                StackValue::Value(Value::Integer(((v1 as u8) / (v2 as u8)).into()))
                            }
                            (false, true, true, Value::Boolean(v1), Value::Integer(v2)) => {
                                StackValue::Value(Value::Integer((v1 as u8) / v2))
                            }
                            (false, true, true, Value::Integer(v1), Value::Boolean(v2)) => {
                                StackValue::Value(Value::Integer(v1 / (v2 as u8)))
                            }
                            (false, true, true, Value::Integer(v1), Value::Integer(v2)) => {
                                StackValue::Value(Value::Integer(v1 / v2))
                            }

                            // %
                            (true, false, false, Value::Boolean(v1), Value::Boolean(v2)) => {
                                StackValue::Value(Value::Integer(((v1 as u8) % (v2 as u8)).into()))
                            }
                            (true, false, false, Value::Boolean(v1), Value::Integer(v2)) => {
                                StackValue::Value(Value::Integer((v1 as u8) % v2))
                            }
                            (true, false, false, Value::Integer(v1), Value::Boolean(v2)) => {
                                StackValue::Value(Value::Integer(v1 % (v2 as u8)))
                            }
                            (true, false, false, Value::Integer(v1), Value::Integer(v2)) => {
                                StackValue::Value(Value::Integer(v1 % v2))
                            }

                            // **
                            (true, false, true, Value::Boolean(v1), Value::Boolean(v2)) => {
                                StackValue::Value(Value::Integer(if !v1 && v2 {
                                    BigInt::zero()
                                } else {
                                    BigInt::one()
                                }))
                            }
                            (true, false, true, Value::Boolean(v1), Value::Integer(v2)) => {
                                StackValue::Value(Value::Integer(
                                    if v1 { BigInt::one() } else { BigInt::zero() }.pow(TryInto::<
                                        BigUint,
                                    >::try_into(
                                        v2
                                    )?),
                                ))
                            }
                            (true, false, true, Value::Integer(v1), Value::Boolean(v2)) => {
                                StackValue::Value(Value::Integer(v1.pow(v2 as u8)))
                            }
                            (true, false, true, Value::Integer(v1), Value::Integer(v2)) => {
                                StackValue::Value(Value::Integer(
                                    v1.pow(TryInto::<BigUint>::try_into(v2)?),
                                ))
                            }

                            _ => return Err(RuntimeError::InvalidInstruction.into()),
                        };
                        stack.push(res);
                    }
                }
            }
            5 => {
                if !instruction[0] {
                    if !instruction[1] {
                        if !instruction[2] {
                            if !instruction[3] {
                                if !instruction[4] {
                                    // swap stack values, 0 is top
                                    let i1: usize = stack
                                        .len()
                                        .checked_sub(Into::<usize>::into(
                                            instructions
                                                .get(ptr + 1)
                                                .ok_or(RuntimeError::InvalidInstruction)?,
                                        ))
                                        .and_then(|x| x.checked_sub(1))
                                        .ok_or(RuntimeError::InvalidInstruction)?;
                                    let i2: usize = stack
                                        .len()
                                        .checked_sub(Into::<usize>::into(
                                            instructions
                                                .get(ptr + 2)
                                                .ok_or(RuntimeError::InvalidInstruction)?,
                                        ))
                                        .and_then(|x| x.checked_sub(1))
                                        .ok_or(RuntimeError::InvalidInstruction)?;
                                    stack.swap(i1, i2);
                                    ptr += 2;
                                } else {
                                    return Err(RuntimeError::InvalidInstruction.into());
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
