//! osu!track v2 backend.  Serves mainly to host the osu!track API endpoints that power the osu!track website and other
//! external applications.  For more information, see README.md in the project root directory.

#![feature(plugin)]
#![plugin(rocket_codegen)]

extern crate chrono;
#[macro_use]
extern crate diesel_codegen;
#[macro_use]
extern crate diesel;
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
extern crate serde_json;
#[macro_use]
extern crate serde_derive;

use chrono::{DateTime, UTC};
use diesel::prelude::*;
use diesel::mysql::MysqlConnection;

mod secret;
use secret::{DbCredentials, DB_CREDENTIALS};
mod routes;
use routes::update;

/// Represents a user.  Maps our internal id to the osu! id and contains the last time the user was updated.
#[derive(Queryable)]
struct User {
    pub id: u32,
    pub osu_id: u32,
    pub username: String,
    pub first_update: DateTime<UTC>,
    pub last_update: DateTime<UTC>,
}

/// Represents an update.  Stored as a snapshot of a player's stats at a certain point in time.
#[derive(Queryable)]
struct Update {
    pub id: u32,
    pub user_id: u32,
    pub mode: u8,
    pub count300: u32,
    pub count100: u32,
    pub count50: u32,
    pub playcount: u32,
    pub ranked_score: u64,
    pub total_score: u64,
    pub pp_rank: u32,
    pub level: f32,
    pub pp_raw: f32,
    pub accuracy: f32,
    pub count_rank_ss: u32,
    pub count_rank_s: u32,
    pub count_rank_a: u32,
    pub update_time: DateTime<UTC>,
}

/// An entry in the beatmap cache.  Holds information about a beatmap in the local database to avoid the delay of querying the osu! API for each one.
#[derive(Queryable)]
struct Beatmap {
    pub id: u32,
    pub mode: u8,
    pub beatmapset_id: u32,
    pub beatmap_id: u32,
    pub approved: u8,
    pub approved_date: DateTime<UTC>,
    pub last_update: DateTime<UTC>,
    pub total_length: u32,
    pub hit_length: u32,
    pub version: String,
    pub artist: String,
    pub title: String,
    pub creator: String,
    pub bpm: u32,
    pub source: String,
    pub difficulty: f32,
    pub diff_size: u32,
    pub diff_overall: u32,
    pub diff_approach: u32,
    pub diff_drain: u32,
}

/// A record of the number of online users in the IRC channel at a given point in time.
#[derive(Queryable)]
struct OnlineUsers {
    pub id: u32,
    pub users: u32,
    pub ops: u32,
    pub voiced: u32,
    pub time_recorded: DateTime<UTC>,
}

/// Represents a hiscore achieved by a user.  Records information about the play, the beatmap, and the time the play occured was achieved and recorded.
#[derive(Queryable)]
struct Hiscore {
    pub id: u32,
    pub user_id: u32,
    pub mode: u32,
    pub beatmap_id: u32,
    pub score: u32,
    pub pp: f32,
    pub mods: u32,
    pub rank: u32,
    pub score_time: DateTime<UTC>,
    pub time_recorded: DateTime<UTC>,
}

pub fn main() {
    let DbCredentials{ host, username, password, database } = DB_CREDENTIALS;
    // setup connection to the database
    let  connection = MysqlConnection::establish(&format!("mysql://{}:{}@{}/{}", username, password, host, database))
        .expect("Unable to connect to MySQL Database!  Please verify your credentials in `secret.rs`");

    // initialize the Rocket webserver
    rocket::ignite().mount("/", routes![routes::update, routes::get_stats]).launch();
}
