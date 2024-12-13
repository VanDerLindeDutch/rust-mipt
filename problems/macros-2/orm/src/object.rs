#![forbid(unsafe_code)]
use std::any::{Any};
use crate::data::DataType;
use crate::storage::{Row, RowSlice};
////////////////////////////////////////////////////////////////////////////////

pub trait Object: Any  {
    fn get_schema(&self) -> Schema;

    fn get_values(&self) -> Row;
    fn get_s()->Schema where Self:Sized;
    fn from_schema(schema: Schema,row: Row) -> Self where Self: Sized;
    // TODO: your code goes here.
}



////////////////////////////////////////////////////////////////////////////////

pub struct Schema {
    pub type_name: &'static str,
    pub table_name: &'static str,
    pub fields: Vec<SchemaField>,
}


pub struct SchemaField {
    pub name: &'static str,
    pub field_type: DataType,
    pub column_name: &'static str,
}





// TODO: your code goes here.
