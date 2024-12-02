#![forbid(unsafe_code)]
use std::any::{Any};
use crate::data::DataType;
////////////////////////////////////////////////////////////////////////////////

pub trait Object: Any  {
    fn get_schema(&self) -> Schema;
    // TODO: your code goes here.
}

////////////////////////////////////////////////////////////////////////////////

pub struct Schema {
    pub type_name: String,
    pub table_name: String,
    pub fields: Vec<SchemaField>,
}


pub struct SchemaField {
    pub name: String,
    pub field_type: DataType,
    pub column_name: String,
}



// TODO: your code goes here.
