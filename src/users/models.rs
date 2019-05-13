use diesel::Queryable;
use rouille::router;
use serde::Deserialize;
use serde::Serialize;
use url::form_urlencoded;

use log::trace;
use log::warn;

use super::schema::users;

use crate::errors::Error;
use crate::errors::ErrorKind;

use crate::HttpMethod;
use crate::Request;

use crate::search::NullableSearch;
use crate::search::Search;

use crate::users::requests;

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

pub enum UserRequest {
    SearchUsers(SearchUser),
    GetUser(u64),
    CreateUser(NewUser),
    UpdateUser(u64, PartialUser),
    DeleteUser(u64),
}

impl UserRequest {
    pub fn from_rouille(
        request: &rouille::Request,
    ) -> Result<UserRequest, Error> {
        let url_queries =
            form_urlencoded::parse(request.raw_query_string().as_bytes());

        router!(request,
            (GET) (/) => {

                let mut first_name_search = Search::NoSearch;
                let mut last_name_search = Search::NoSearch;
                let mut banner_id_search = Search::NoSearch;
                let mut email_search = NullableSearch::NoSearch;

                for (field, query) in url_queries {
                    match field.as_ref() {
                        "first_name" => first_name_search = Search::from_query(query.as_ref())?,
                        "last_name" => last_name_search = Search::from_query(query.as_ref())?,
                        "banner_id" => banner_id_search = Search::from_query(query.as_ref())?,
                        "email" => email_search = NullableSearch::from_query(query.as_ref())?,
                        _ => return Err(Error::new(ErrorKind::Url)),
                    }
                }

                Ok(UserRequest::SearchUsers(SearchUser {
                    first_name: first_name_search,
                    last_name: last_name_search,
                    banner_id: banner_id_search,
                    email: email_search,
                }))
            },

            (GET) (/{id: u64}) => {
                Ok(UserRequest::GetUser(id))
            },

            (POST) (/) => {
                let request_body = request.data().ok_or(Error::new(ErrorKind::Body))?;
                let new_user: NewUser = serde_json::from_reader(request_body)?;

                Ok(UserRequest::CreateUser(new_user))
            },

            (POST) (/{id: u64}) => {
                let request_body = request.data().ok_or(Error::new(ErrorKind::Body))?;
                let update_user: PartialUser = serde_json::from_reader(request_body)?;

                Ok(UserRequest::UpdateUser(id, update_user))
            },

            (DELETE) (/{id: u64}) => {
                Ok(UserRequest::DeleteUser(id))
            },

            _ => {
                warn!("Could not create a user request for the given rouille request");
                Err(Error::new(ErrorKind::NotFound))
            }
        )
    }
}

impl Request for UserRequest {
    fn from_parts<
        'a,
        P: Iterator<Item = &'a str>,
        Q: Iterator<Item = (&'a str, &'a str)>,
        B: std::io::Read,
    >(
        method: HttpMethod,
        mut path: P,
        query: Q,
        body: B,
    ) -> Result<Self, Error> {
        match (method, path.next().map(|p| p.parse())) {
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

                Ok(UserRequest::SearchUsers(SearchUser {
                    first_name: first_name_search,
                    last_name: last_name_search,
                    banner_id: banner_id_search,
                    email: email_search,
                }))
            }

            (HttpMethod::GET, Some(Ok(id))) => Ok(UserRequest::GetUser(id)),

            (HttpMethod::POST, None) => {
                let new_user: NewUser = serde_json::from_reader(body)?;
                Ok(UserRequest::CreateUser(new_user))
            }

            (HttpMethod::POST, Some(Ok(id))) => {
                let update_user: PartialUser = serde_json::from_reader(body)?;
                Ok(UserRequest::UpdateUser(id, update_user))
            }

            (HttpMethod::DELETE, Some(Ok(id))) => {
                Ok(UserRequest::DeleteUser(id))
            }

            _ => Err(Error::new(ErrorKind::Url)),
        }
    }
}

pub enum UserResponse {
    OneUser(User),
    ManyUsers(UserList),
    NoResponse,
}

impl UserResponse {
    pub fn to_rouille(self) -> rouille::Response {
        match self {
            UserResponse::OneUser(user) => rouille::Response::json(&user),
            UserResponse::ManyUsers(users) => rouille::Response::json(&users),
            UserResponse::NoResponse => rouille::Response::empty_204(),
        }
    }
}
