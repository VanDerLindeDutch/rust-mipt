#![forbid(unsafe_code)]
use crate::{
    data::{DataType, Value},
    error::{Error, Result, UnexpectedTypeError},
    object::Schema,
    ObjectId,
};
use rusqlite::{types::FromSqlError, OptionalExtension, ToSql};
use std::{borrow::Cow, fmt::Write};
use crate::error::{ErrorCtx, ErrorWithCtx, MissingColumnError, NotFoundError};
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
        let res: rusqlite::Result<i32> = self.query_row("SELECT 1 FROM sqlite_master WHERE name = ?1", [table], |x: &rusqlite::Row| x.get(0));
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
        let mut sql = format!("CREATE TABLE {table_name} ( id INTEGER PRIMARY KEY AUTOINCREMENT, ");
        let mut row_names = String::new();
        for field in &schema.fields {
            sql.push_str(&(field.column_name.to_owned() + " " + field.field_type.sqllite_repr() + ", "))
        }

            sql.remove(sql.len() - 1);
            sql.remove(sql.len() - 1);


        sql.push_str(" );");
        println!("{}", sql);
        self.execute(&sql, [])?;
        Ok(())
    }

    fn insert_row(&self, schema: &Schema, row: &RowSlice) -> Result<ObjectId> {
        if !self.table_exists(schema.table_name)? {
            self.create_table(schema)?;
        }
        let table_name = &schema.table_name;

        let mut row_names = String::new();
        let mut values = String::new();
        let mut params: Vec<&dyn ToSql> = vec![];
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
                    println!("{:?}", val);
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
                    } else {
                        ""
                    };
                    row_names.push_str(delimiter);
                    values.push_str(delimiter);
                    params.push(val.get_as_sql())
                }
            }
        }
        let mut sql = format!("INSERT INTO {table_name} ({row_names}) VALUES ({values})");
        if row.is_empty() {
            sql = format!("INSERT INTO {table_name} DEFAULT VALUES")
        }
        if let Err(err) = self.execute(&sql, &params[..]) {
            let mut ctx = ErrorCtx::default();
            ctx.schema = Some(schema);
            return Err(ErrorWithCtx::new_error(err, ctx));
        }
        Ok(ObjectId(self.last_insert_rowid()))
    }

    fn update_row(&self, id: ObjectId, schema: &Schema, row: &RowSlice) -> Result<()> {
        let table_name = &schema.table_name;

        let mut row_names = String::with_capacity(row.len() * 7);
        let mut params: Vec<&dyn ToSql> = vec![];
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
                    row_names.push_str(" = ?");
                    let delimiter = if i != row.len() - 1 {
                        ", "
                    } else {
                        ""
                    };
                    row_names.push_str(delimiter);
                    params.push(val.get_as_sql())
                }
            }
        }
        let sql = format!("UPDATE {table_name} SET {row_names} WHERE id = ?");
        params.push(&id.0);
        if let Err(err) = self.execute(&sql, &params[..]) {
            let mut ctx = ErrorCtx::default();
            ctx.schema = Some(schema);
            ctx.object_id = Some(id);
            return Err(ErrorWithCtx::new_error(err, ctx));
        }

        Ok(())
    }

    fn select_row(&self, id: ObjectId, schema: &Schema) -> Result<Row<'static>> {
        let mut out = Row::new();
        let values = &schema.fields;
        let res: rusqlite::Result<()> = self.query_row(&format!("SELECT * FROM {}  WHERE id = ?1", schema.table_name), [&id.0.to_string()], |x: &rusqlite::Row| {
            for item in values.iter().enumerate() {
                let i = item.0 + 1;
                match item.1.field_type {
                    DataType::String => {
                        let sql_item = x.get::<_, String>(i)?;

                        out.push(Value::String(Cow::from(sql_item)))
                    }
                    DataType::Bytes => {
                        let sql_item = x.get::<_, Vec<u8>>(i)?;
                        out.push(Value::Bytes(Cow::from(sql_item)))
                    }
                    DataType::Int64 => {
                        let sql_item = x.get::<_, i64>(i)?;
                        out.push(Value::Int64(sql_item))
                    }
                    DataType::Float64 => {
                        let sql_item = x.get::<_, f64>(i)?;
                        out.push(Value::Float64(sql_item))
                    }
                    DataType::Bool => {
                        let sql_item = x.get::<_, bool>(i)?;
                        out.push(Value::Bool(sql_item))
                    }
                }
            }
            Ok(())
        });
        if let Err(err) = res {
            let mut ctx = ErrorCtx::default();
            ctx.schema = Some(schema);
            ctx.object_id = Some(id);
            return Err(ErrorWithCtx::new_error(err, ctx));
        }
        Ok(out)
    }

    fn delete_row(&self, id: ObjectId, schema: &Schema) -> Result<()> {
        let table_name = schema.table_name;
        let sql = format!("DELETE FROM {table_name} WHERE id = ?");

        self.execute(&sql, [id.0])?;
        Ok(())
    }

    fn commit(&self) -> Result<()> {
        let res = self.execute("COMMIT;", []);
        match res {
            Ok(_) => { Ok(()) }
            Err(e) => {
                Err(match_sqllite_error(e))
            }
        }
    }

    fn rollback(&self) -> Result<()> {
        let res = self.execute("ROLLBACK;", []);
        match res {
            Ok(_) => { Ok(()) }
            Err(e) => {
                Err(match_sqllite_error(e))
            }
        }
    }
}
fn match_sqllite_error(error: rusqlite::Error) -> Error {
    match error {
        rusqlite::Error::SqliteSingleThreadedMode => {
            Error::LockConflict
        }
        rusqlite::Error::SqliteFailure(e, _) => Error::Storage(Box::new(e)),
        _ => panic!("{}", error)
    }
}

fn match_sqllite_get_error(error: rusqlite::Error, s: &Schema, id: ObjectId) -> Error {
    match error {
        rusqlite::Error::SqliteSingleThreadedMode => {
            Error::LockConflict
        }
        rusqlite::Error::SqliteFailure(e, _) => Error::Storage(Box::new(e)),
        rusqlite::Error::QueryReturnedNoRows => Error::NotFound(Box::new(NotFoundError { object_id: id, type_name: s.type_name })),
        rusqlite::Error::InvalidColumnType(i, col, t) => Error::UnexpectedType(Box::new(UnexpectedTypeError {
            type_name: s.type_name,
            attr_name: String::leak(col.clone()),
            table_name: s.table_name,
            column_name: String::leak(col),
            expected_type: s.fields[i - 1].field_type,
            got_type: t.to_string(),
        })),
        _ => panic!("{}", error)
    }
}