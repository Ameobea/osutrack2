//! Maps the API endpoints to functions

use std::time::SystemTime;

use chrono::NaiveDateTime;
use rocket_contrib::JSON;

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
    pub update_time: NaiveDateTime,
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

/// Updates a user's stats using the osu! API and returns the changes since the last recorded update.
#[get("/update/<username>/<mode>")]
pub fn update(username: &str, mode: u32) -> JSON<UpdateDiff> {
    // return JSON(UpdateDiff {})
    unimplemented!(); // TODO
}

/// Returns current static statistics for a user
#[get("/stats/<username>/<mode>")]
pub fn get_stats(username: &str, mode: u32) -> Option<JSON<Stats>> {
    unimplemented!(); // TODO
}
