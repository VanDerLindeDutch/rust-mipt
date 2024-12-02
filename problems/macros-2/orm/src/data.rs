#![forbid(unsafe_code)]
use std::{borrow::Cow};
use std::any::{Any, TypeId};
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct ObjectId(i64);

// TODO: your code goes here.

////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DataType {
    String,
    Bytes,
    Int64,
    Float64,
    Bool,
}

////////////////////////////////////////////////////////////////////////////////

pub enum Value<'a> {
    String(Cow<'a, str>),
    Bytes(Cow<'a, [u8]>),
    Int64(i64),
    Float64(f64),
    Bool(bool),
}


impl DataType {
    pub fn from<T: 'static>() -> Self {

        let t = TypeId::of::<T>();
        match t {

            _ if t == TypeId::of::<String>() => DataType::String,
            _ if t == TypeId::of::<Vec<u8>>() => DataType::Bytes,
            _ if t == TypeId::of::<i64>() => DataType::Int64,
            _ if t == TypeId::of::<f64>() => DataType::Float64,
            _ if t ==TypeId::of::<bool>() => DataType::Bool,
            _ => {unimplemented!()}
        }
    }

    pub fn sqllite_repr(&self) -> &str {
        match self {
            DataType::String => {
                "TEXT"
            }
            DataType::Bytes => {
                "BLOB"
            }
            DataType::Int64 => {
                "BIGINT"
            }
            DataType::Float64 => {
                "REAL"
            }
            DataType::Bool => {
                "TINYINT"
            }
        }
    }
}

impl PartialEq<Value<'_>> for DataType {
    fn eq(&self, other: &Value) -> bool {
        match (self, other) {
            (DataType::String, Value::String(_)) => true,
            (DataType::Bytes, Value::Bytes(_)) => true,
            (DataType::Int64, Value::Int64(_)) => true,
            (DataType::Float64, Value::Float64(_)) => true,
            (DataType::Bool, Value::Bool(_)) => true,
            (_, _) => false
        }
    }
}