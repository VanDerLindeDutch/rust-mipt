#![forbid(unsafe_code)]
use crate::{
    data::{DataType, Value},
    error::{Error,  Result, UnexpectedTypeError},
    object::Schema,
    ObjectId,
};
use rusqlite::{types::FromSqlError, OptionalExtension, ToSql};
use std::{borrow::Cow, fmt::Write};
use std::any::Any;
use crate::error::MissingColumnError;
////////////////////////////////////////////////////////////////////////////////

pub type Row<'a> = Vec<Value<'a>>;
pub type RowSlice<'a> = [Value<'a>];

////////////////////////////////////////////////////////////////////////////////

pub(crate) trait StorageTransaction {
    fn table_exists(&self, table: &str) -> Result<bool>;
    fn create_table(&self, schema: &Schema) -> Result<()>;

    fn insert_row(&self, schema: &Schema, row: &RowSlice) -> Result<ObjectId>;
    fn update_row(&self, id: ObjectId, schema: &Schema, row: &RowSlice) -> Result<()>;
    fn select_row(&self, id: ObjectId, schema: &Schema) -> Result<Row<'static>>;
    fn delete_row(&self, id: ObjectId, schema: &Schema) -> Result<()>;

    fn commit(&self) -> Result<()>;
    fn rollback(&self) -> Result<()>;
}

impl<'a> StorageTransaction for rusqlite::Transaction<'a> {
    fn table_exists(&self, table: &str) -> Result<bool> {
        let res = self.query_row("SELECT 1 FROM sqlite_master WHERE name = ?1", [table], |x| x.get(0));
        match res {
            Err(e) => {
                if e.eq(&rusqlite::Error::QueryReturnedNoRows) {
                    Ok(false)
                } else {
                    Err(match_sqllite_error(e))
                }
            }
            Ok(_) => {
                Ok(true)
            }
        }
    }

    fn create_table(&self, schema: &Schema) -> Result<()> {
        let table_name = &schema.table_name;
        let mut sql = format!("CREATE TABLE {table_name} (id INTEGER PRIMARY KEY AUTOINCREMENT,");
        let mut row_names = String::new();
        for field in schema.fields {
            sql.push_str(&(field.column_name + " " + field.field_type.sqllite_repr() + ", "))
        }
        self.execute(&sql, [])?;
        Ok(())
    }

    fn insert_row(&self, schema: &Schema, row: &RowSlice) -> Result<ObjectId> {
        let table_name = &schema.table_name;

        let mut row_names = String::new();
        let mut values = String::new();
        let mut params: Vec<Cow<dyn Any>> = vec![];
        for (i, field) in schema.fields.iter().enumerate() {
            match row.get(i) {
                None => {
                    return Err(Error::MissingColumn(Box::new(MissingColumnError {
                        type_name: "",
                        attr_name: "",
                        table_name: "",
                        column_name: "",
                    })));
                }
                Some(val) => {
                    if !field.field_type.eq(val) {
                        return Err(Error::UnexpectedType(Box::new(UnexpectedTypeError {
                            type_name: "",
                            attr_name: "",
                            table_name: "",
                            column_name: "",
                            expected_type: DataType::String,
                            got_type: "".to_string(),
                        })));
                    }
                    row_names.push_str(&field.column_name);
                    values.push_str("?");
                    let delimiter = if i != row.len() - 1 {
                        ", "
                    }else {
                        ""
                    };
                    row_names.push_str(delimiter);
                    values.push_str(delimiter);
                    match val {
                        Value::String(v) |
                        Value::Bytes(v) |
                        Value::Int64(v) |
                        Value::Float64(v) |
                        Value::Bool(v)
                        => { params.push(v.clone()); }
                    }
                }
            }
        }
        let sql = format!("INSERT INTO {table_name} ({row_names}) VALUES ({values})");
        self.execute(&sql, rusqlite::params![params])?;
        Ok(ObjectId(self.last_insert_rowid()))
    }

    fn update_row(&self, id: ObjectId, schema: &Schema, row: &RowSlice) -> Result<()> {
        // TODO: your code goes here.
        unimplemented!()
    }

    fn select_row(&self, id: ObjectId, schema: &Schema) -> Result<Row<'static>> {
        // TODO: your code goes here.
        unimplemented!()
    }

    fn delete_row(&self, id: ObjectId, schema: &Schema) -> Result<()> {
        // TODO: your code goes here.
        unimplemented!()
    }

    fn commit(&self) -> Result<()> {
        // TODO: your code goes here.
        unimplemented!()
    }

    fn rollback(&self) -> Result<()> {
        // TODO: your code goes here.
        unimplemented!()
    }
}
fn match_sqllite_error(error: rusqlite::Error) -> Error {
    match error {
        rusqlite::Error::SqliteSingleThreadedMode => {
            Error::LockConflict
        }
        rusqlite::Error::SqliteFailure(e, _) => Error::Storage(Box::new(e)),
        _ => panic!()
    }
}