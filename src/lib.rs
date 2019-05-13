#[macro_use]
extern crate diesel;

#[macro_use]
extern crate diesel_migrations;

pub mod access;
pub mod errors;
pub mod search;
pub mod users;
pub mod chemicals;

use serde::Serialize;

use errors::Error;

pub enum HttpMethod {
    GET,
    POST,
    DELETE,
}

pub trait Request: Sized {
    fn from_parts<
        'a,
        P: Iterator<Item = &'a str>,
        Q: Iterator<Item = (&'a str, &'a str)>,
        B: std::io::Read
    >(
        method: HttpMethod,
        path: P,
        query: Q,
        body: B,
    ) -> Result<Self, Error>;
}

pub trait Response {
    fn to_parts<S: Serialize>(self) -> Result<S, Error>;
}

