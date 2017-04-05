//! osu!track v2 backend.  Serves mainly to host the osu!track API endpoints that power the osu!track website and other
//! external applications.  For more information, see README.md in the project root directory.

#[macro_use]
extern crate diesel_codegen;
#[macro_use]
extern crate diesel;
extern crate rocket;

use std::time::SystemTime;

mod secret;
use secret::DB_CREDENTIALS;

/// Represents a user.  Maps our internal id to the osu! id and contains the last time the user was updated.
#[derive(Queryable)]
struct User {
    pub id: i32,
    pub osu_id: i32,
    pub username: String,
    last_update: SystemTime
}

/// Represents an update.  Stored as a snapshot of a player's stats at a certain point in time.
#[derive(Queryable)]
struct Update {
    pub id: i32,
    pub user_id: i32,
    pub count300: i32,
    pub count100: i32,
    pub count50: i32,
    pub playcount: i32,
    pub ranked_score: i32,
    pub total_score: i32,
    pub pp_rank: i32,
    pub level: f32,
    pub pp_raw: f32,
    pub accuracy: f32,
    pub count_rank_ss: i32,
    pub count_rank_s: i32,
    pub count_rank_a: i32,
    pub update_time: SystemTime,
    pub mode: i32,
}

/// An entry in the beatmap cache.  Holds information about a beatmap in the local database to avoid the delay of querying the osu! API for each one.
#[derive(Queryable)]
struct Beatmap {
    pub id: i32,
    pub mode: i32,
    pub beatmapset_id: i32,
    pub beatmap_id: i32,
    pub approved: i32,
    pub approved_date: SystemTime,
    pub last_update: SystemTime,
    pub total_length: i32,
    pub hit_length: i32,
    pub version: String,
    pub artist: String,
    pub title: String,
    pub creator: String,
    pub bpm: i32,
    pub source: String,
    pub difficulty: f32,
    pub diff_size: i32,
    pub diff_overall: i32,
    pub diff_approach: i32,
    pub diff_drain: i32,
}

/// A record of the number of online users in the IRC channel at a given point in time.
#[derive(Queryable)]
struct OnlineUsers {
    pub id: i32,
    pub users: i32,
    pub ops: i32,
    pub voiced: i32,
    pub time_recorded: SystemTime,
}

/// Represents a hiscore achieved by a user.  Records information about the play, the beatmap, and the time the play occured was achieved and recorded.
#[derive(Queryable)]
struct Hiscore {
    pub id: i32,
    pub user_id: i32,
    pub mode: i32,
    pub beatmap_id: i32,
    pub score: i32,
    pub pp: f32,
    pub mods: i32,
    pub rank: i32,
    pub score_time: SystemTime,
    pub time_recorded: SystemTime,
}

pub fn main() {
    rocket::ignite();
}
