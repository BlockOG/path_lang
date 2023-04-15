use std::{iter::Peekable, ops::Index, str::Chars};

use anyhow::Result;
use num_bigint::{BigUint, ToBigUint};
use thiserror::Error;

#[derive(Default, Debug, Clone)]
pub(crate) struct Instruction {
    op: Vec<bool>,
}

impl Instruction {
    pub(crate) fn add_part(&mut self, part: &str) {
        match part {
            "." => self.op.push(false),
            ".." => self.op.push(true),
            _ => panic!("Invalid instruction part \"{}\"", part),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.op.is_empty()
    }

    pub(crate) fn len(&self) -> usize {
        self.op.len()
    }

    pub(crate) fn get(&self, index: usize) -> Option<bool> {
        self.op.get(index).copied()
    }
}

#[derive(Debug, Error)]
pub(crate) enum ParseInstructionError {
    #[error("Instruction not finished")]
    UnfinishedInstruction,
}

pub(crate) enum ParsedInstruction {
    Instruction(Instruction),
    Err(ParseInstructionError),
    CodePartStopped,
}

pub(crate) trait ParseInstruction {
    fn parse_instruction(&mut self) -> ParsedInstruction;
}

impl ParseInstruction for Peekable<Chars<'_>> {
    fn parse_instruction(&mut self) -> ParsedInstruction {
        if self.peek().is_none() {
            return ParsedInstruction::CodePartStopped;
        }

        let mut instruction = Instruction::default();
        let mut part = String::new();

        for i in self {
            match (part.len(), i) {
                (0, '/') => break,
                (0 | 1, '.') => part.push(i),
                (1 | 2, '/') => {
                    instruction.add_part(&part);
                    part.clear();
                }
                _ => {
                    if instruction.is_empty() {
                        return ParsedInstruction::CodePartStopped;
                    } else {
                        return ParsedInstruction::Err(
                            ParseInstructionError::UnfinishedInstruction,
                        );
                    }
                }
            }
        }

        ParsedInstruction::Instruction(instruction)
    }
}

impl Index<usize> for Instruction {
    type Output = bool;

    fn index(&self, index: usize) -> &Self::Output {
        &self.op[index]
    }
}

impl From<&Instruction> for BigUint {
    fn from(instruction: &Instruction) -> Self {
        let mut result = 0.to_biguint().unwrap();
        for (i, bit) in instruction.op.iter().enumerate() {
            if *bit {
                result += 1.to_biguint().unwrap() << i;
            }
        }
        result
    }
}

impl From<&Instruction> for bool {
    fn from(instruction: &Instruction) -> Self {
        instruction.op[0]
    }
}

impl From<&Instruction> for Result<u8> {
    fn from(instruction: &Instruction) -> Self {
        if instruction.len() != 8 {
            return Err(anyhow::anyhow!("Invalid instruction length"));
        }
        let mut result = 0;
        for (i, bit) in instruction.op.iter().rev().enumerate() {
            if *bit {
                result += 1 << i;
            }
        }
        Ok(result)
    }
}
