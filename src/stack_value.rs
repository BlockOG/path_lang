use std::cmp::Ordering;

use num_bigint::{BigInt, BigUint};

use crate::value::Value;

#[derive(Clone, PartialEq)]
pub(crate) enum StackValue {
    Value(Value),
    Argument(Value),
    Optional(BigUint, Value),
}

impl From<Value> for StackValue {
    fn from(value: Value) -> Self {
        StackValue::Value(value)
    }
}

impl From<BigInt> for StackValue {
    fn from(value: BigInt) -> Self {
        StackValue::Value(Value::Integer(value))
    }
}

impl From<BigUint> for StackValue {
    fn from(value: BigUint) -> Self {
        StackValue::Value(Value::Integer(value.into()))
    }
}

impl From<bool> for StackValue {
    fn from(value: bool) -> Self {
        StackValue::Value(Value::Boolean(value))
    }
}

impl From<String> for StackValue {
    fn from(value: String) -> Self {
        StackValue::Value(Value::String(value))
    }
}

impl From<char> for StackValue {
    fn from(value: char) -> Self {
        StackValue::Value(Value::Integer((value as u32).into()))
    }
}

impl PartialOrd<StackValue> for StackValue {
    fn partial_cmp(&self, other: &StackValue) -> Option<Ordering> {
        match (self, other) {
            (StackValue::Value(v1), StackValue::Value(v2)) => v1.partial_cmp(v2),
            (StackValue::Argument(v1), StackValue::Argument(v2)) => v1.partial_cmp(v2),
            (StackValue::Optional(_, v1), StackValue::Optional(_, v2)) => v1.partial_cmp(v2),
            _ => None,
        }
    }
}
