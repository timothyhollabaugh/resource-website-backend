use diesel;
use diesel::mysql::MysqlConnection;
use diesel::query_builder::AsQuery;
use diesel::ExpressionMethods;
use diesel::QueryDsl;
use diesel::RunQueryDsl;
use diesel::TextExpressionMethods;

use log::trace;
use log::warn;

use crate::HttpMethod;

use crate::errors::Error;
use crate::errors::ErrorKind;

use crate::search::NullableSearch;
use crate::search::Search;

use crate::users::models::{
    NewUser, PartialUser, SearchUser, User, UserList
};
use crate::users::schema::users as users_schema;

pub fn handle_user(
    method: HttpMethod,
    mut path: Vec<String>,
    query: Vec<(String, String)>,
    body: String,
    database_connection: &MysqlConnection,
) -> Result<Option<String>, Error> {
    match (method, path.pop().map(|p| p.parse())) {
        (HttpMethod::GET, None) => {
            let mut first_name_search = Search::NoSearch;
            let mut last_name_search = Search::NoSearch;
            let mut banner_id_search = Search::NoSearch;
            let mut email_search = NullableSearch::NoSearch;

            for (field, query) in query {
                match field.as_ref() {
                    "first_name" => {
                        first_name_search =
                            Search::from_query(query.as_ref())?
                    }
                    "last_name" => {
                        last_name_search =
                            Search::from_query(query.as_ref())?
                    }
                    "banner_id" => {
                        banner_id_search =
                            Search::from_query(query.as_ref())?
                    }
                    "email" => {
                        email_search =
                            NullableSearch::from_query(query.as_ref())?
                    }
                    _ => return Err(Error::new(ErrorKind::Url)),
                }
            }

            let response = search_users(SearchUser {
                first_name: first_name_search,
                last_name: last_name_search,
                banner_id: banner_id_search,
                email: email_search,
            }, database_connection)?;

            Ok(Some(serde_json::to_string(&response)?))
        }

        (HttpMethod::GET, Some(Ok(id))) => {
            let response = get_user(id, database_connection)?;
            Ok(Some(serde_json::to_string(&response)?))
        }

        (HttpMethod::POST, None) => {
            let new_user: NewUser = serde_json::from_str(&body)?;
            let response = create_user(new_user, database_connection)?;
            Ok(Some(serde_json::to_string(&response)?))
        }

        (HttpMethod::POST, Some(Ok(id))) => {
            let new_user: PartialUser = serde_json::from_str(&body)?;
            update_user(id, new_user, database_connection)?;
            Ok(None)
        }

        (HttpMethod::DELETE, Some(Ok(id))) => {
            delete_user(id, database_connection)?;
            Ok(None)
        }

        _ => Err(Error::new(ErrorKind::Url)),
    }
}

fn search_users(
    user: SearchUser,
    database_connection: &MysqlConnection,
) -> Result<UserList, Error> {
    let mut users_query = users_schema::table.as_query().into_boxed();

    match user.first_name {
        Search::Partial(s) => {
            users_query = users_query
                .filter(users_schema::first_name.like(format!("%{}%", s)))
        }

        Search::Exact(s) => {
            users_query = users_query.filter(users_schema::first_name.eq(s))
        }

        Search::NoSearch => {}
    }

    match user.last_name {
        Search::Partial(s) => {
            users_query = users_query
                .filter(users_schema::last_name.like(format!("%{}%", s)))
        }

        Search::Exact(s) => {
            users_query = users_query.filter(users_schema::last_name.eq(s))
        }

        Search::NoSearch => {}
    }

    match user.banner_id {
        Search::Partial(s) => {
            warn!("Trying to partial search by banner id. This is not currently supported, so performing exact search instead");
            trace!("Partial search required the field to be a text field, but banner id is currently an integet");;
            users_query = users_query.filter(users_schema::banner_id.eq(s))
        }

        Search::Exact(s) => {
            users_query = users_query.filter(users_schema::banner_id.eq(s))
        }

        Search::NoSearch => {}
    }

    match user.email {
        NullableSearch::Partial(s) => {
            users_query =
                users_query.filter(users_schema::email.like(format!("%{}%", s)))
        }

        NullableSearch::Exact(s) => {
            users_query = users_query.filter(users_schema::email.eq(s))
        }

        NullableSearch::Some => {
            users_query = users_query.filter(users_schema::email.is_not_null());
        }

        NullableSearch::None => {
            users_query = users_query.filter(users_schema::email.is_null());
        }

        NullableSearch::NoSearch => {}
    }

    let found_users = users_query.load::<User>(database_connection)?;
    let user_list = UserList { users: found_users };

    Ok(user_list)
}

fn get_user(
    id: u64,
    database_connection: &MysqlConnection,
) -> Result<User, Error> {
    let mut found_users = users_schema::table
        .filter(users_schema::id.eq(id))
        .load::<User>(database_connection)?;

    match found_users.pop() {
        Some(user) => Ok(user),
        None => Err(Error::new(ErrorKind::NotFound)),
    }
}

fn create_user(
    user: NewUser,
    database_connection: &MysqlConnection,
) -> Result<User, Error> {
    diesel::insert_into(users_schema::table)
        .values(user)
        .execute(database_connection)?;

    let mut inserted_users = users_schema::table
        .filter(diesel::dsl::sql("id = LAST_INSERT_ID()"))
        .load::<User>(database_connection)?;

    if let Some(inserted_user) = inserted_users.pop() {
        Ok(inserted_user)
    } else {
        Err(Error::new(ErrorKind::Database))
    }
}

fn update_user(
    id: u64,
    user: PartialUser,
    database_connection: &MysqlConnection,
) -> Result<(), Error> {
    diesel::update(users_schema::table)
        .filter(users_schema::id.eq(id))
        .set(&user)
        .execute(database_connection)?;
    Ok(())
}

fn delete_user(
    id: u64,
    database_connection: &MysqlConnection,
) -> Result<(), Error> {
    diesel::delete(users_schema::table.filter(users_schema::id.eq(id)))
        .execute(database_connection)?;

    Ok(())
}
