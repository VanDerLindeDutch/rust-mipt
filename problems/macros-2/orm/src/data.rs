#![forbid(unsafe_code)]
use std::{borrow::Cow};
use std::any::{Any, TypeId};
use std::fmt::{Display, Formatter};
use rusqlite::ToSql;
use rusqlite::types::FromSql;
////////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Copy, PartialEq, Eq, Debug, Hash)]
pub struct ObjectId(pub i64);

impl ObjectId {
    pub fn into_i64(self) -> i64 {
        self.0
    }
}

impl From<i64> for ObjectId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}
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
#[derive(Debug)]
pub enum Value<'a> {
    String(Cow<'a, str>),
    Bytes(Cow<'a, [u8]>),
    Int64(i64),
    Float64(f64),
    Bool(bool),
}

impl Value<'_> {
    pub fn from_data_type(value: &dyn Any, data_type: DataType) -> Value<'_> {
        match data_type {
            DataType::String => {
                let val = value.downcast_ref::<String>().unwrap();
                Value::String(Cow::from(val))
            }
            DataType::Bytes => {
                let val = value.downcast_ref::<Vec<u8>>().unwrap();
                Value::Bytes(Cow::from(val))
            }
            DataType::Int64 => {
                let val = value.downcast_ref::<i64>().unwrap();
                Value::Int64(*val)
            }
            DataType::Float64 => {
                let val = value.downcast_ref::<f64>().unwrap();
                Value::Float64(*val)
            }
            DataType::Bool => {
                let val = value.downcast_ref::<bool>().unwrap();
                Value::Bool(*val)
            }
        }
    }


    pub fn get_as_sql(&self) -> &dyn ToSql {
        match self {
            Value::String(v) => {
                v
            }
            Value::Bytes(v) => {
                v
            }
            Value::Int64(v) => {
                v
            }
            Value::Float64(v) => {
                v
            }
            Value::Bool(v) => {
                v
            }
        }
    }

}

impl<'a> From<&Value<'a>> for String  {
    fn from(value: &Value) -> Self {
        match value {
            Value::String(v) => {
                String::from(v.clone())
            }
            _ => {panic!()}
        }
    }
}

impl<'a> From<&Value<'a>> for Vec<u8>  {
    fn from(value: &Value) -> Self {
        match value {
            Value::Bytes(v) => {
                Vec::from(v.clone())
            }
            _ => {panic!()}
        }
    }
}
impl<'a> From<&Value<'a>> for i64  {
    fn from(value: &Value) -> Self {
        match value {
            Value::Int64(v) => {
                *v
            }
            _ => {panic!()}
        }
    }
}
impl<'a> From<&Value<'a>> for f64  {
    fn from(value: &Value) -> Self {
        match value {
            Value::Float64(v) => {
                *v
            }
            _ => {panic!()}
        }
    }
}

impl<'a> From<&Value<'a>> for bool  {
    fn from(value: &Value) -> Self {
        match value {
            Value::Bool(v) => {
                *v
            }
            _ => {panic!()}
        }
    }
}
impl DataType {
    pub fn from<T: 'static>() -> Self {
        let t = TypeId::of::<T>();
        match t {
            _ if t == TypeId::of::<String>() => DataType::String,
            _ if t == TypeId::of::<Vec<u8>>() => DataType::Bytes,
            _ if t == TypeId::of::<i64>() => DataType::Int64,
            _ if t == TypeId::of::<f64>() => DataType::Float64,
            _ if t == TypeId::of::<bool>() => DataType::Bool,
            _ => { unimplemented!() }
        }
    }

    pub fn to_get(&self) -> TypeId {
        self.type_id()
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