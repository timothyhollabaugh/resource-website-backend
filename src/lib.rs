#[macro_use]
extern crate diesel;
extern crate diesel_migrations;

pub mod access;
pub mod chemicals;
pub mod errors;
pub mod search;
pub mod tests;
pub mod users;

use diesel::MysqlConnection;
use diesel::mysql::Mysql;
use diesel::Queryable;
use diesel::query_source::Table as DieselTable;
use diesel::query_builder::AsQuery;
use diesel::RunQueryDsl;
use diesel::query_dsl::methods::LoadQuery;

use crate::errors::Error;

struct ItemList<T> {
    items: Vec<T>,
}

trait Model: Sized {
    type Table: AsQuery + DieselTable + LoadQuery<MysqlConnection, Self::DbModel>;
    type SqlType;
    type DbModel: Queryable<<<Self as Model>::Table as AsQuery>::SqlType, Mysql>;

    fn from_db(db: Self::DbModel) -> Self;
    fn to_db(self) -> Self::DbModel;

    fn table() -> Self::Table;

    fn read_all(database_connection: &MysqlConnection) -> Result<ItemList<Self>, Error> {
        // Load the db items from the database
        let db_items = Self::table().load::<Self::DbModel>(database_connection)?;

        // Convert the db items into real items
        let items = db_items.into_iter().map(|db| Self::from_db(db)).collect();

        Ok(ItemList { items })
    }
}
