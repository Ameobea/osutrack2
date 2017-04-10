//! Maps the API endpoints to functions

use chrono::NaiveDateTime;
use diesel;
use diesel::prelude::*;
use rocket::State;
use rocket_contrib::JSON;

use super::DbPool;
use helpers::debug;
use models::{Update, NewUpdate, Hiscore, NewHiscore};
use osu_api::ApiClient;
use schema::updates::dsl as updates_dsl;

/// Holds the changes between two updates
#[derive(Serialize)]
pub struct UpdateDiff {
    pub first_update: bool,
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
    pub newhs: Vec<NewHiscore>,
}

impl UpdateDiff {
    /// Given two different updates, returns a new `UpdateDiff` representing the difference between them.  If the first
    /// update doesn't exist, then the first update will be treated as containing all zeros.
    pub fn diff(prev: Option<&Update>, cur: &NewUpdate) -> UpdateDiff {
        match prev {
            Some(prev) => UpdateDiff {
                first_update: false,
                count300: cur.count300 - prev.count300,
                count100: cur.count100 - prev.count100,
                count50: cur.count50 - prev.count50,
                playcount: cur.playcount - prev.playcount,
                ranked_score: cur.ranked_score - prev.ranked_score,
                total_score: cur.total_score - prev.total_score,
                pp_rank: cur.pp_rank - prev.pp_rank,
                level: cur.level - prev.level,
                pp_raw: cur.pp_raw - prev.pp_raw,
                accuracy: cur.accuracy - prev.accuracy,
                count_rank_ss: cur.count_rank_ss - prev.count_rank_ss,
                count_rank_s: cur.count_rank_s - prev.count_rank_s,
                count_rank_a: cur.count_rank_a - prev.count_rank_a,
                pp_country_rank: cur.pp_country_rank - prev.pp_country_rank,
            },
            None => UpdateDiff {
                first_update: true,
                count300: cur.count300,
                count100: cur.count100,
                count50: cur.count50,
                playcount: cur.playcount,
                ranked_score: cur.ranked_score,
                total_score: cur.total_score,
                pp_rank: cur.pp_rank,
                level: cur.level,
                pp_raw: cur.pp_raw,
                accuracy: cur.accuracy,
                count_rank_ss: cur.count_rank_ss,
                count_rank_s: cur.count_rank_s,
                count_rank_a: cur.count_rank_a,
                pp_country_rank: cur.pp_country_rank,
            }
        }
    }
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
pub fn update(
    api_client: State<ApiClient>, db_pool: State<DbPool>, username: &str, mode: u8
) -> Result<Option<JSON<UpdateDiff>>, String> {
    let client = api_client.inner();
    let db_conn = &*db_pool.get_conn();

    let stats = client.get_stats(username, mode)?;
    match stats {
        None => { return Ok(None); },
        Some(s) => {
            // find the most recent update in the same game mode
            let last_update: Vec<Update> = updates_dsl::updates
                .filter(updates_dsl::user_id.eq(s.user_id))
                .filter(updates_dsl::mode.eq(mode as i16))
                .order(updates_dsl::id.desc())
                .limit(1)
                .load::<Update>(db_conn)
                .map_err(debug)?;

            // if there was a change worth recording between the two updates, write it to the database
            if last_update.len() > 0 {
                let first = last_update.first().unwrap();
                if first.pp_rank != s.pp_rank || s.playcount != s.playcount || s.pp_country_rank != s.pp_country_rank {
                    diesel::insert(&s)
                        .into(updates_dsl::updates)
                        .execute(db_conn)
                        .map_err(debug)?;
                }
            }

            unimplemented!(); // TODO: Integrate hiscores

            // calculate the difference between the current stats and the last update (if it exists) and return them
            UpdateDiff::diff(last_update.first(), &s);
        }
    }
}

/// Returns current static statistics for a user
#[get("/stats/<username>/<mode>")]
pub fn get_stats(api_client: State<ApiClient>, username: &str, mode: u8) -> Option<JSON<Stats>> {
    unimplemented!(); // TODO
}

/// Returns the difference between a user's current stats and the last time they earned PP
#[get("/lastpp/<username>/<mode>")]
pub fn get_last_pp_diff(api_client: State<ApiClient>, username: &str, mode: u8) -> Option<JSON<UpdateDiff>> {
    unimplemented!();
}
