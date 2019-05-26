#[macro_use]
extern crate diesel;
extern crate diesel_migrations;

pub mod access;
pub mod chemicals;
pub mod errors;
pub mod search;
pub mod tests;
pub mod users;

use diesel::mysql::Mysql;
use diesel::query_builder::AsQuery;
use diesel::query_source::Table as DieselTable;
use diesel::MysqlConnection;
use diesel::Queryable;
use diesel::RunQueryDsl;
//use diesel::QueryDsl;
use diesel::expression::AsExpression;
use diesel::expression::Expression;
use diesel::expression_methods::ExpressionMethods;
use diesel::helper_types::Eq as DieselEq;
use diesel::helper_types::Filter;
use diesel::query_dsl::filter_dsl::FilterDsl;
use diesel::query_dsl::methods::LoadQuery;

use serde::Deserialize;
use serde::Serialize;

use crate::errors::Error;
use crate::errors::ErrorKind;

#[derive(Serialize, Deserialize)]
pub struct ItemList<T> {
    items: Vec<T>,
}

trait DbBase: Sized {
    type Table: AsQuery
        + DieselTable
        + LoadQuery<MysqlConnection, Self::DbModel>;
    //type Id: Default + Clone + Copy + Expression + SelectableExpression + Column;
    type SqlType;
    type DbModel: Queryable<
        <<Self as DbBase>::Table as AsQuery>::SqlType,
        Mysql,
    >;

    fn from_db(db: Self::DbModel) -> Self;
    fn into_db(self) -> Self::DbModel;

    fn table() -> Self::Table;
}

trait DbReadAll: DbBase {
    fn read_all(
        database_connection: &MysqlConnection,
    ) -> Result<ItemList<Self>, Error> {
        // Load the db items from the database
        let db_items =
            Self::table().load::<Self::DbModel>(database_connection)?;

        // Convert the db items into real items
        let items = db_items.into_iter().map(|db| Self::from_db(db)).collect();

        Ok(ItemList { items })
    }
}

trait DbReadSingle: DbBase {
    fn read_single<ID>(
        id: ID,
        database_connection: &MysqlConnection,
    ) -> Result<Self, Error>
    where
        ID: AsExpression<
            <<Self::Table as DieselTable>::PrimaryKey as Expression>::SqlType,
        >,

        <<Self as DbBase>::Table as DieselTable>::PrimaryKey : ExpressionMethods,

        <Self as DbBase>::Table:
            FilterDsl<DieselEq<<Self::Table as DieselTable>::PrimaryKey, ID>>,

        Filter<
            <Self as DbBase>::Table,
            DieselEq<<Self::Table as DieselTable>::PrimaryKey, ID>
        >: LoadQuery<MysqlConnection, Self::DbModel>
    {
        // Load the db items from the database
        let table = Self::table();
        let filter = table.filter(Self::table().primary_key().eq(id));
        let db_items = filter.load::<Self::DbModel>(database_connection)?;

        // Convert the db items into real items
        let mut items: Vec<_> = db_items.into_iter().map(|db| Self::from_db(db)).collect();

        if let Some(item) = items.pop() {
            Ok(item)
        } else {
            Err(Error::new(ErrorKind::NotFound))
        }
    }
}
