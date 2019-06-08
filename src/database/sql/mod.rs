// TODO: Make use of diesel::r2d2 support for connection pooling.

use crate::database;
use database::models::{
    Create, DeleteById, Exercise, ExerciseDao, FindById, NewExercise, Update, UpdatedExercise,
};
use database::Error::SqlError;
use database::Result;
use diesel::backend::{Backend, SupportsDefaultKeyword, UsesAnsiSavepointSyntax};
use diesel::prelude::*;
use diesel::query_dsl::UpdateAndFetchResults;
use schema::*;

/// Auto-generated module created by Diesel from the schema defined by the migrations in
/// "migrations/" for the purpose of constructing and validating SQL queries at compile-time.
///
/// See <http://diesel.rs/guides/schema-in-depth/>.
pub mod schema;

// NOTE: Generic type DB must have an explicit lifetime to ensure that any values that contain
// references to <Conn as Connection>::Backend do not outlive any references in the DB type itself.
//
// To the best of my knowledge, all diesel types that implement diesel::backend::Backend do not
// contain references and, to the borrow checker, are indistinguishable from types whose references
// all have static lifetimes. Thus, the "DB: 'static" lifetime bound is trivially satisfied by all
// diesel Backend implementations.
//
// See https://doc.rust-lang.org/book/ch19-02-advanced-lifetimes.html#lifetime-bounds-on-references-to-generic-types.

impl<'a, Conn, DB: 'static> ExerciseDao for Conn
where
    Conn: for<'b> FindById<&'b str, Exercise>,
    Conn: for<'b> UpdateAndFetchResults<&'b UpdatedExercise<'b>, Exercise>,
    Conn: Connection<Backend = DB>,
    DB: Backend<RawValue = [u8]>,
    DB: SupportsDefaultKeyword,
    DB: UsesAnsiSavepointSyntax,
{
}

impl<'a, Conn, DB: 'static> Create<&'a NewExercise<'a>, Exercise> for Conn
where
    Conn: for<'b> FindById<&'b str, Exercise>,
    Conn: Connection<Backend = DB>,
    DB: Backend,
    DB: SupportsDefaultKeyword,
{
    fn create(&self, obj: &NewExercise) -> Result<Exercise> {
        diesel::insert_into(exercises::table)
            .values(obj)
            .execute(self)
            .map_err(SqlError)?;

        self.find_by_id(obj.get_id())
    }
}

impl<'a, Conn, DB: 'static> FindById<&'a str, Exercise> for Conn
where
    Conn: Connection<Backend = DB>,
    DB: Backend<RawValue = [u8]>,
    DB: UsesAnsiSavepointSyntax,
{
    fn find_by_id(&self, id: &'a str) -> Result<Exercise> {
        exercises::table.find(id).first(self).map_err(SqlError)
    }
}

impl<'a, Conn, DB: 'static> Update<&'a UpdatedExercise<'a>, Exercise> for Conn
where
    Conn: Connection<Backend = DB>,
    Conn: for<'b> UpdateAndFetchResults<&'b UpdatedExercise<'b>, Exercise>,
    DB: Backend,
    DB: SupportsDefaultKeyword,
{
    fn update(&self, obj: &'a UpdatedExercise<'a>) -> Result<Exercise> {
        obj.save_changes(self).map_err(SqlError)
    }
}

impl<'a, Conn, DB: 'static> DeleteById<&'a str, Exercise> for Conn
where
    Conn: Connection<Backend = DB>,
    Conn: for<'b> FindById<&'b str, Exercise>,
    DB: Backend<RawValue = [u8]>,
    DB: UsesAnsiSavepointSyntax,
{
    fn delete_by_id(&self, id: &'a str) -> Result<Exercise> {
        let exercise = self.find_by_id(id);
        diesel::delete(exercises::table.find(id))
            .execute(self)
            .map_err(SqlError)
            .and_then(|_| exercise)
    }
}
