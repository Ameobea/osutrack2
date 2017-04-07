//! osu!track v2 backend.  Serves mainly to host the osu!track API endpoints that power the osu!track website and other
//! external applications.  For more information, see README.md in the project root directory.

#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate chrono;
#[macro_use]
extern crate diesel_codegen;
#[macro_use]
extern crate diesel;
extern crate hyper;
extern crate hyper_native_tls;
#[macro_use]
extern crate lazy_static;
extern crate log;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use r2d2::{ GetTimeout, Pool, PooledConnection, Config };
use r2d2_diesel::ConnectionManager;
use rocket::http::Status;
use rocket::Request;
use rocket::Outcome::{Success, Failure};
use rocket::request::{FromRequest, Outcome};

mod secret;
use secret::DB_CREDENTIALS;
mod routes;
use routes::update;
mod schema;
mod models;
mod osu_api;
mod helpers;
use helpers::create_db_pool;

pub struct DB(PooledConnection<ConnectionManager<MysqlConnection>>);

impl DB {
    pub fn conn(&self) -> &MysqlConnection {
        &*self.0
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for DB {
    type Error = GetTimeout;
    fn from_request(_: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        match DB_POOL.get() {
            Ok(conn) => Success(DB(conn)),
            Err(e) => Failure((Status::InternalServerError, e)),
        }
    }
}

lazy_static! {
    pub static ref DB_POOL: Pool<ConnectionManager<MysqlConnection>> = create_db_pool();
}

pub fn main() {
    // initialize the Rocket webserver
    rocket::ignite().mount("/", routes![routes::update, routes::get_stats]).launch();
}
