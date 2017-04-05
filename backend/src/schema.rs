//! Definitions of data types that are stored in the database or retrieved from the osu! API

use chrono::{DateTime, UTC};

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

/// Holds the changes between two updates
#[derive(Serialize)]
pub struct UpdateDiff {
    pub first_update: bool,
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
    pub update_time: DateTime<UTC>,
    pub mode: i32,
}

/// A snapshot of a User's statistics at a specific point in time
#[derive(Serialize)]
pub struct Stats {
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
}
