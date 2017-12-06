//! osu!track v2 backend.  Serves mainly to host the osu!track API endpoints that power the osu!track website and other
//! external applications.  For more information, see README.md in the project root directory.

#![feature(plugin)]
#![plugin(rocket_codegen)]
#![recursion_limit="256"]

extern crate chrono;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_infer_schema;
extern crate log;
extern crate r2d2;
extern crate r2d2_diesel;
extern crate reqwest;
extern crate rocket;
// #[macro_use]
extern crate rocket_contrib;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use diesel::mysql::MysqlConnection;
use r2d2::{ Pool, PooledConnection };
use r2d2_diesel::ConnectionManager;

mod secret;
mod routes;
mod schema;
mod models;
mod osu_api;
use osu_api::ApiClient;
mod helpers;
use helpers::create_db_pool;

pub struct DbPool(Pool<ConnectionManager<MysqlConnection>>);

impl DbPool {
    pub fn get_conn(&self) -> PooledConnection<ConnectionManager<MysqlConnection>> {
        return self.0.get().unwrap()
    }
}

pub fn main() {
    // initialize the Rocket webserver
    rocket::ignite()
        .mount("/", routes![
            routes::update, routes::get_stats, routes::get_last_pp_diff, routes::live_stats, routes::get_updates,
            routes::get_hiscores, routes::get_beatmaps, routes::get_beatmap,
        ])
        .manage(ApiClient::new())
        .manage(DbPool(create_db_pool()))
        .launch();
}
