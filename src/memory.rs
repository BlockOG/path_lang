use std::collections::HashMap;

use num_bigint::{BigUint, ToBigUint};

use crate::{
    function::{BuiltInFunction, Function},
    Value,
};

pub(crate) struct Memory {
    memory: HashMap<BigUint, Value>,
}

impl Memory {
    pub(crate) fn new() -> Memory {
        let mut memory = HashMap::new();

        memory.insert(
            0.to_biguint().unwrap(),
            Value::Function(Function::new_built_in(true, 0, BuiltInFunction::Print)),
        );
        memory.insert(
            1.to_biguint().unwrap(),
            Value::Function(Function::new_built_in(true, 0, BuiltInFunction::PrintLn)),
        );

        Memory { memory }
    }

    pub(crate) fn get(&self, index: BigUint) -> Option<&Value> {
        self.memory.get(&index)
    }

    pub(crate) fn set(&mut self, index: BigUint, value: Value) {
        self.memory.insert(index, value);
    }
}
