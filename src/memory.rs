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
        // memory.insert(
        //     2.to_biguint().unwrap(),
        //     Value::Function(Function::new_built_in(false, 1, BuiltInFunction::Read)),
        // );
        memory.insert(
            3.to_biguint().unwrap(),
            Value::Function(Function::new_built_in(false, 0, BuiltInFunction::ReadLn)),
        );
        memory.insert(
            4.to_biguint().unwrap(),
            Value::Function(Function::new_built_in(false, 1, BuiltInFunction::ToBool)),
        );
        memory.insert(
            5.to_biguint().unwrap(),
            Value::Function(Function::new_built_in(false, 1, BuiltInFunction::ToStr)),
        );
        memory.insert(
            6.to_biguint().unwrap(),
            Value::Function(Function::new_built_in(false, 1, BuiltInFunction::ToInt)),
        );
        memory.insert(
            7.to_biguint().unwrap(),
            Value::Function(Function::new_built_in(false, 1, BuiltInFunction::Trim)),
        );
        memory.insert(
            8.to_biguint().unwrap(),
            Value::Function(Function::new_built_in(false, 1, BuiltInFunction::Len)),
        );
        memory.insert(
            9.to_biguint().unwrap(),
            Value::Function(Function::new_built_in(false, 2, BuiltInFunction::Push)),
        );
        memory.insert(
            10.to_biguint().unwrap(),
            Value::Function(Function::new_built_in(false, 1, BuiltInFunction::Pop)),
        );

        Memory { memory }
    }

    pub(crate) fn get(&self, index: BigUint) -> Option<&Value> {
        self.memory.get(&index)
    }

    pub(crate) fn set(&mut self, index: BigUint, value: Value) {
        self.memory.insert(index, value);
    }

    pub(crate) fn remove(&mut self, index: BigUint) {
        self.memory.remove(&index);
    }
}
