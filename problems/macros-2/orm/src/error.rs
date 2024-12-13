#![forbid(unsafe_code)]


use crate::{data::DataType, object::Schema, ObjectId};
use thiserror::Error;
use crate::object::SchemaField;
////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    NotFound(Box<NotFoundError>),
    #[error(transparent)]
    UnexpectedType(Box<UnexpectedTypeError>),
    #[error(transparent)]
    MissingColumn(Box<MissingColumnError>),
    #[error("database is locked")]
    LockConflict,
    #[error("storage error: {0}")]
    Storage(#[source] Box<dyn std::error::Error>),
}

impl<'a> From<ErrorWithCtx<'a, rusqlite::Error>> for Error {
    fn from(err: ErrorWithCtx<'a, rusqlite::Error>) -> Self {
        match err.val {
            rusqlite::Error::QueryReturnedNoRows => {
                Error::NotFound(Box::new(NotFoundError { object_id: err.ctx.object_id.unwrap(), type_name: err.ctx.schema.unwrap().type_name }))
            }
            rusqlite::Error::SqliteFailure(inner_err, Some(v)) => {

                if v.contains("no such column") || v.contains("has no column named")  {
                    let ErrorCtx { schema: Some(Schema { type_name, table_name, .. }), .. } = err.ctx else { unimplemented!() };
                    let SchemaField { column_name, field_type: expected_type, name: attr_name } = err.ctx.schema.unwrap().fields[0];
                    Error::MissingColumn(Box::new(MissingColumnError { type_name, attr_name, table_name, column_name }))
                } else if v.contains("no such table"){
                    Error::NotFound(Box::new(NotFoundError { object_id: err.ctx.object_id.unwrap(), type_name: err.ctx.schema.unwrap().type_name }))
                } else if v.contains("database is locked"){
                    Error::LockConflict
                }else {
                    Error::Storage(Box::new(inner_err))
                }

            }

            rusqlite::Error::InvalidColumnType(i, _, got_type) => {
                let ErrorCtx { schema: Some(Schema { type_name, table_name, .. }), .. } = err.ctx else { unimplemented!() };
                let SchemaField { column_name, field_type: expected_type, name: attr_name } = err.ctx.schema.unwrap().fields[i - 1];
                Error::UnexpectedType(Box::new(UnexpectedTypeError {
                    type_name,
                    attr_name,
                    table_name,
                    column_name,
                    expected_type,
                    got_type: got_type.to_string(),
                }))
            }
            rusqlite::Error::InvalidColumnIndex(i) => {
                let ErrorCtx { schema: Some(Schema { type_name, table_name, .. }), .. } = err.ctx else { unimplemented!() };
                let SchemaField { column_name,  name: attr_name, .. } = err.ctx.schema.unwrap().fields[i - 1];
                Error::MissingColumn(Box::new(MissingColumnError{
                    type_name,
                    attr_name,
                    table_name,
                    column_name,
                }))
            }
            err => { Error::Storage(Box::new(err)) }
        }
    }
}

impl From<rusqlite::Error> for Error {
    fn from(err: rusqlite::Error) -> Self {
        Self::from(ErrorWithCtx::new(err, ErrorCtx::default()))
    }
}

#[derive(Default)]
pub struct ErrorCtx<'a> {
    pub object_id: Option<ObjectId>,
    pub schema: Option<&'a Schema>,
    pub attr_name: Option<&'a str>,
}

impl From<&Schema> for ErrorCtx<'_> {
    fn from(value: &Schema) -> Self {
        todo!()
    }
}

pub struct ErrorWithCtx<'a, T: std::error::Error>
{
    val: T,
    ctx: ErrorCtx<'a>,
}

impl<'a, T: std::error::Error> ErrorWithCtx<'a, T> {
    pub fn new(val: T, ctx: ErrorCtx<'a>) -> Self {
        Self { val, ctx }
    }

    pub fn new_error(val: T, ctx: ErrorCtx<'a>) -> Error
    where
        Error: From<ErrorWithCtx<'a, T>>,
    {
        Error::from(Self { val, ctx })
    }
}


////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
#[error("object is not found: type '{type_name}', id {object_id}")]
pub struct NotFoundError {
    pub object_id: ObjectId,
    pub type_name: &'static str,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
#[error(
    "invalid type for {type_name}::{attr_name}: expected equivalent of {expected_type:?}, \
    got {got_type} (table: {table_name}, column: {column_name})"
)]
pub struct UnexpectedTypeError {
    pub type_name: &'static str,
    pub attr_name: &'static str,
    pub table_name: &'static str,
    pub column_name: &'static str,
    pub expected_type: DataType,
    pub got_type: String,
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Error, Debug)]
#[error(
    "missing a column for {type_name}::{attr_name} \
    (table: {table_name}, column: {column_name})"
)]
pub struct MissingColumnError {
    pub type_name: &'static str,
    pub attr_name: &'static str,
    pub table_name: &'static str,
    pub column_name: &'static str,
}

////////////////////////////////////////////////////////////////////////////////

pub type Result<T> = std::result::Result<T, Error>;
