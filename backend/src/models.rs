//! Definitions of data types that are stored in the database or retrieved from the osu! API

use chrono::NaiveDateTime;
use schema::{users, updates, hiscores, beatmaps, online_users};

/// Represents a user.  Maps our internal id to the osu! id and contains the last time the user was updated.
#[derive(Associations, Identifiable, Queryable)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub first_update: NaiveDateTime,
    pub last_update: NaiveDateTime,
}

/// A new user, ready to be inserted into the database.  Maps usernames to osu_ids and holds metadata about the first and most
/// recent times the user was updated.
#[derive(Insertable)]
#[table_name="users"]
pub struct NewUser {
    pub id: i32,
    pub username: String,
}

/// Represents an update for a user containing a snapshot of their stats at a certain point in time.
#[derive(Associations, Clone, Identifiable, Serialize, Queryable)]
#[belongs_to(User)]
pub struct Update {
    pub id: i32,
    pub user_id: i32,
    pub mode: i16,
    pub count300: i32,
    pub count100: i32,
    pub count50: i32,
    pub playcount: i32,
    pub ranked_score: i64,
    pub total_score: i64,
    pub pp_rank: i32,
    pub level: f32,
    pub pp_raw: f32,
    pub accuracy: f32,
    pub count_rank_ss: i32,
    pub count_rank_s: i32,
    pub count_rank_a: i32,
    pub pp_country_rank: i32,
    pub update_time: NaiveDateTime,
}

/// Represents a current snapshot of a user's statistics ready to be inserted in the database.
#[derive(Associations, Clone, Debug, Insertable, Serialize)]
#[table_name="updates"]
pub struct NewUpdate {
    pub user_id: i32,
    pub mode: i16,
    pub count300: i32,
    pub count100: i32,
    pub count50: i32,
    pub playcount: i32,
    pub ranked_score: i64,
    pub total_score: i64,
    pub pp_rank: i32,
    pub level: f32,
    pub pp_raw: f32,
    pub accuracy: f32,
    pub count_rank_ss: i32,
    pub count_rank_s: i32,
    pub count_rank_a: i32,
    pub pp_country_rank: i32,
}

/// An entry in the beatmap cache.  Holds information about a beatmap in the local database to avoid the delay of querying the osu! API for each one.
#[derive(Clone, Debug, Deserialize, Insertable, Queryable, Serialize)]
#[table_name = "beatmaps"]
pub struct Beatmap {
    pub mode: i16,
    pub beatmapset_id: i32,
    pub beatmap_id: i32,
    pub approved: i16,
    pub approved_date: NaiveDateTime,
    pub last_update: NaiveDateTime,
    pub total_length: i32,
    pub hit_length: i32,
    pub version: String,
    pub artist: String,
    pub title: String,
    pub creator: String,
    pub bpm: f32,
    pub source: String,
    pub difficulty: f32,
    pub diff_size: f32,
    pub diff_overall: f32,
    pub diff_approach: f32,
    pub diff_drain: f32,
}

/// A record of the number of online users in the IRC channel at a given point in time.
#[derive(Queryable)]
pub struct OnlineUsers {
    pub time_recorded: NaiveDateTime,
    pub users: i32,
    pub operators: i32,
    pub voiced: i32,
}

/// A new recording of the number of currently online users, ready to be inserted into the database.
#[derive(Insertable)]
#[table_name="online_users"]
pub struct NewOnlineUsers {
    pub users: i32,
    pub operators: i32,
    pub voiced: i32,
}

/// Represents a hiscore achieved by a user.  Records information about the play, the beatmap, and the time the play occured was achieved and recorded.
#[derive(Associations, Queryable, Serialize)]
#[belongs_to(User)]
pub struct Hiscore {
    pub id: i32,
    pub user_id: i32,
    pub mode: i16,
    pub beatmap_id: i32,
    pub score: i32,
    pub pp: f32,
    pub enabled_mods: i32,
    pub rank: String,
    pub score_time: NaiveDateTime,
    pub time_recorded: NaiveDateTime,
}

/// Represents a new hiscore set by a user, ready to be inserted into the database.
#[derive(Insertable, Serialize)]
#[table_name="hiscores"]
pub struct NewHiscore {
    pub user_id: i32,
    pub mode: i16,
    pub beatmap_id: i32,
    pub score: i32,
    pub pp: f32,
    pub enabled_mods: i32,
    pub rank: String,
    pub score_time: NaiveDateTime,
}
