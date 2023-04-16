use std::fmt::{self, Display};

use num_bigint::{BigInt, BigUint};

use crate::{function::Function, stack_value::StackValue};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Value {
    Boolean(bool),
    Integer(BigInt),
    // Float(BigFloat),
    String(String),
    Array(Vec<Value>),
    Function(Function),
}

impl Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Integer(i) => write!(f, "{}", i),
            // Value::Float(fl) => write!(f, "{}", fl),
            Value::String(s) => write!(f, "{}", s),
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", v)?;
                }
                write!(f, "]")
            }
            Value::Function(fu) => write!(f, "{}", fu),
        }
    }
}

impl From<Value> for BigInt {
    fn from(value: Value) -> Self {
        match value {
            Value::Boolean(b) => if b { 1 } else { 0 }.into(),
            Value::Integer(i) => i,
            // Value::Float(fl) => fl.to_bigint().unwrap(),
            Value::String(s) => s.parse().unwrap(),
            Value::Array(_) => panic!("Can't convert array to integer"),
            Value::Function(_) => panic!("Can't convert function to integer"),
        }
    }
}

impl From<Value> for bool {
    fn from(value: Value) -> Self {
        match value {
            Value::Boolean(b) => b,
            Value::Integer(i) => i != 0.into(),
            // Value::Float(fl) => fl != 0.into(),
            Value::String(s) => !s.is_empty(),
            Value::Array(a) => !a.is_empty(),
            Value::Function(_) => panic!("Can't convert function to boolean"),
        }
    }
}

impl From<BigInt> for Value {
    fn from(value: BigInt) -> Self {
        Value::Integer(value)
    }
}

impl From<BigUint> for Value {
    fn from(value: BigUint) -> Self {
        Value::Integer(value.into())
    }
}

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Boolean(value)
    }
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.to_string())
    }
}

impl From<Function> for Value {
    fn from(value: Function) -> Self {
        Value::Function(value)
    }
}

impl PartialOrd<Value> for Value {
    fn partial_cmp(&self, other: &Value) -> Option<std::cmp::Ordering> {
        match (self, other) {
            (Value::Boolean(b1), Value::Boolean(b2)) => b1.partial_cmp(b2),
            (Value::Integer(i1), Value::Integer(i2)) => i1.partial_cmp(i2),
            // (Value::Float(f1), Value::Float(f2)) => f1.partial_cmp(f2),
            (Value::String(s1), Value::String(s2)) => s1.partial_cmp(s2),
            (Value::Array(a1), Value::Array(a2)) => a1.partial_cmp(a2),
            _ => None,
        }
    }
}

impl From<StackValue> for Value {
    fn from(value: StackValue) -> Self {
        match value {
            StackValue::Value(v) => v,
            StackValue::Argument(v) => v,
            StackValue::Optional(_, v) => v,
        }
    }
}
