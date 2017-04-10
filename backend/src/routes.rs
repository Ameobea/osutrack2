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
use schema::hiscores::dsl as hiscores_dsl;

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
    pub fn diff(prev: Option<&Update>, cur: &NewUpdate, old_hs: Vec<Hiscore>, new_hs: Vec<NewHiscore>) -> UpdateDiff {
        match prev {
            Some(prev) => {
                // find hiscores that are in the new hiscores but not the old hiscores
                let hs_diff: Vec<NewHiscore> = new_hs.into_iter().filter_map(|cur_hs| -> Option<NewHiscore> {
                    let mut is_duplicate = false;
                    for old_hs in &old_hs {
                        if old_hs.beatmap_id == cur_hs.beatmap_id && old_hs.score == cur_hs.score {
                            is_duplicate = true;
                            break;
                        }
                    }

                    if is_duplicate { None } else { Some(cur_hs) }
                }).collect();

                UpdateDiff {
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
                    newhs: hs_diff,
                }
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
                newhs: new_hs
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
            let needs_insert = if last_update.len() > 0 {
                let first = last_update.first().unwrap();
                first.pp_rank != s.pp_rank ||
                    s.playcount != s.playcount ||
                    s.pp_country_rank != s.pp_country_rank
            } else {
                true
            };

            if needs_insert {
                diesel::insert(&s)
                    .into(updates_dsl::updates)
                    .execute(db_conn)
                    .map_err(debug)?;
            }

            // look up the user's previous hiscores
            let old_hiscores: Vec<Hiscore> = hiscores_dsl::hiscores
                .filter(hiscores_dsl::user_id.eq(s.user_id))
                .filter(hiscores_dsl::mode.eq(mode as i16))
                .load::<Hiscore>(db_conn)
                .map_err(debug)?;

            // get the user's current hiscores
            let cur_hiscores = match api_client.get_user_best(s.user_id, mode, 100)? {
                Some(hs) => hs,
                None => Vec::new(),
            };

            // calculate the diff between the last and current updates
            let diff = UpdateDiff::diff(last_update.first(), &s, old_hiscores, cur_hiscores);

            // insert all new hiscores into the database
            diesel::insert(&diff.newhs)
                .into(hiscores_dsl::hiscores)
                .execute(db_conn)
                .map_err(debug)?;

            // TODO: Prefetch all of the beatmaps and update them into the cache

            // calculate the difference between the current stats and the last update (if it exists) and return them
            Ok(Some(JSON(diff)))
        }
    }
}

/// Returns current static statistics for a user
#[get("/stats/<username>/<mode>")]
pub fn get_stats(api_client: State<ApiClient>, username: &str, mode: u8) -> Option<JSON<Stats>> {
    unimplemented!(); // TODO
}

/// Returns the difference between a user's current stats and the last time their total PP score was different than its
/// current value.
#[get("/lastpp/<username>/<mode>")]
pub fn get_last_pp_diff(
    api_client: State<ApiClient>, db_pool: State<DbPool>, username: &str, mode: u8
) -> Result<Option<JSON<UpdateDiff>>, String> {
    let client = api_client.inner();
    let db_conn = &*db_pool.get_conn();

    let stats = client.get_stats(username, mode)?;
    match stats {
        None => { return Ok(None); },
        Some(s) => {
            // find the most recent update in the same game mode where `pp_raw` is different than current.
            let last_different_update: Vec<Update> = updates_dsl::updates
                .filter(updates_dsl::user_id.eq(s.user_id))
                .filter(updates_dsl::mode.eq(mode as i16))
                .filter(updates_dsl::pp_raw.ne(s.pp_raw))
                .order(updates_dsl::id.desc())
                .limit(1)
                .load::<Update>(db_conn)
                .map_err(debug)?;
            let last_different_update = if last_different_update.len() > 0 { Some(&last_different_update[0]) } else { None };

            // find the first recorded update that has the same pp as the user currently does
            let first_same_update_time: Option<NaiveDateTime> = if last_different_update.is_some() {
                let same_updates = updates_dsl::updates
                    .filter(updates_dsl::id.gt(last_different_update.unwrap().id))
                    .filter(updates_dsl::user_id.eq(s.user_id))
                    .filter(updates_dsl::mode.eq(mode as i16))
                    .order(updates_dsl::id.asc())
                    .select(updates_dsl::update_time)
                    .limit(1)
                    .load::<NaiveDateTime>(db_conn)
                    .map_err(debug)?;
                if same_updates.len() > 0 {
                    Some(same_updates[0].clone())
                } else {
                    None
                }
            } else {
                None
            };

            // get the user's current hiscores
            let cur_hiscores: Vec<NewHiscore> = match api_client.get_user_best(s.user_id, mode, 100)? {
                Some(hs) => hs,
                None => Vec::new(),
            };

            let old_hiscores: Vec<Hiscore> = if last_different_update.is_some() {
                // look up the user's hiscores that were made before the last significant update
                let query = hiscores_dsl::hiscores
                    .filter(hiscores_dsl::user_id.eq(s.user_id))
                    .filter(hiscores_dsl::mode.eq(mode as i16));

                // if there were updates previous to this with the same pp value, pick the earliest one and
                // enforce a bound that all hiscores were recorded previous to it.
                if first_same_update_time.is_some() {
                    query.filter(hiscores_dsl::time_recorded.lt(first_same_update_time.unwrap()))
                        .load::<Hiscore>(db_conn)
                        .map_err(debug)?

                } else {
                    query.load::<Hiscore>(db_conn)
                        .map_err(debug)?
                }
            } else {
                // there has been no update for the user where there pp is different than it currently is,
                // so simply report all of their hiscores as new
                Vec::new()
            };

            // calculate the diff between the current and last significant update and return it
            Ok(Some(JSON(UpdateDiff::diff(last_different_update, &s, old_hiscores, cur_hiscores))))
        }
    }
}
