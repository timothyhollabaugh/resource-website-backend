use diesel::Queryable;
use serde::Deserialize;
use serde::Serialize;

use super::schema::users;

use crate::search::NullableSearch;
use crate::search::Search;

#[derive(Queryable, Serialize, Deserialize)]
pub struct User {
    pub id: u64,
    pub first_name: String,
    pub last_name: String,
    pub banner_id: u32,
    pub email: Option<String>,
}

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[table_name = "users"]
pub struct NewUser {
    pub first_name: String,
    pub last_name: String,
    pub banner_id: u32,
    pub email: Option<String>,
}

#[derive(AsChangeset, Serialize, Deserialize)]
#[table_name = "users"]
pub struct PartialUser {
    pub first_name: Option<String>,
    pub last_name: Option<String>,
    pub banner_id: Option<u32>,
    pub email: Option<Option<String>>,
}

pub struct SearchUser {
    pub first_name: Search<String>,
    pub last_name: Search<String>,
    pub banner_id: Search<u32>,
    pub email: NullableSearch<String>,
}

#[derive(Serialize, Deserialize)]
pub struct UserList {
    pub users: Vec<User>,
}

