//! Maps the API endpoints to functions

use std::collections::HashMap;

use chrono::NaiveDateTime;
use diesel;
use diesel::prelude::*;
use diesel::BelongingToDsl;
use rocket::State;
use rocket_contrib::Json;
use serde_json;

use super::DbPool;
use helpers::{debug, get_user_from_username, get_last_update};
use models::{Beatmap, Update, NewUpdate, Hiscore, NewHiscore, User};
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

/// Updates a user's stats using the osu! API and returns the changes since the last recorded update.
#[get("/update/<username>/<mode>")]
pub fn update(
    api_client: State<ApiClient>, db_pool: State<DbPool>, username: String, mode: u8
) -> Result<Option<Json<UpdateDiff>>, String> {
    let client = api_client.inner();
    let db_conn = &*db_pool.get_conn();

    let stats = client.get_stats(&username, mode)?;
    match stats {
        None => { return Ok(None); },
        Some(s) => {
            let last_update: Option<Update> = get_last_update(s.user_id, mode, db_conn)?;

            // if there was a change worth recording between the two updates, write it to the database
            let needs_insert = if last_update.is_some() {
                let first = last_update.as_ref().unwrap();
                first.pp_rank != s.pp_rank ||
                    s.playcount != s.playcount ||
                    s.pp_country_rank != s.pp_country_rank
            } else {
                true
            };

            if needs_insert {
                diesel::insert_into(updates_dsl::updates)
                    .values(&s)
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
            let diff = UpdateDiff::diff(last_update.as_ref(), &s, old_hiscores, cur_hiscores);

            // insert all new hiscores into the database
            diesel::insert_into(hiscores_dsl::hiscores)
                .values(&diff.newhs)
                .execute(db_conn)
                .map_err(debug)?;

            // TODO: Prefetch all of the beatmaps and update them into the cache

            // calculate the difference between the current stats and the last update (if it exists) and return them
            Ok(Some(Json(diff)))
        }
    }
}

/// Returns current static statistics for a user as stored in the osu!track database.  Designed to be extrememly fast and
/// avoid the osu! server round-trip involved with getting live stats.  Returns a 404 if there is no stored updates for the
/// user in the selected mode.
#[get("/stats/<username>/<mode>")]
pub fn get_stats(db_pool: State<DbPool>, username: String, mode: u8) -> Result<Option<Json<Update>>, String> {
    let db_conn = &*db_pool.get_conn();

    let usr: User = match get_user_from_username(db_conn, &username)? {
        Some(usr) => usr,
        None => { return Ok(None); },
    };

    Update::belonging_to(&usr)
        .order(updates_dsl::id.desc())
        .filter(updates_dsl::mode.eq(mode as i16))
        .first(db_conn)
        .map(|x| Some(Json(x)))
        .map_err(debug)
}

/// Returns the live view of a user's stats as reported by the osu! API.  Functions the same way as the `/update/` endpoint
/// but returns the current statistics rather than the change since the last update
#[get("/livestats/<username>/<mode>")]
pub fn live_stats(
    api_client: State<ApiClient>, db_pool: State<DbPool>, username: String, mode: u8
) -> Result<Option<Json<NewUpdate>>, String> {
    let client = api_client.inner();
    let db_conn = &*db_pool.get_conn();

    let stats: NewUpdate = match client.get_stats(&username, mode)? {
        Some(u) => u,
        None => { return Ok(None); },
    };

    // check to see if the user exists in our database yet.  If it doesn't, it will soon because the `get_stats()`
    // function inserts it on another thread.
    let usr: User = match get_user_from_username(db_conn, &username)? {
        Some(usr) => usr,
        None => {
            // this means that the DB is currently in the process of inserting the user and update, so we don't need to bother
            return Ok(Some(Json(stats)));
        },
    };

    // find the last stored update for the user and, if there has been a change, insert a new update
    let last_update = get_last_update(usr.id, mode, db_conn)?;

    // if there was a change worth recording between the two updates, write it to the database
    let needs_insert = if last_update.is_some() {
        let first = last_update.unwrap();
        first.pp_rank != stats.pp_rank ||
            stats.playcount != stats.playcount ||
            stats.pp_country_rank != stats.pp_country_rank
    } else {
        true
    };

    if needs_insert {
        diesel::insert_into(updates_dsl::updates)
            .values(&stats)
            .execute(db_conn)
            .map_err(debug)?;
    }

    Ok(Some(Json(stats)))
}

/// Returns all of a user's stored updates for a given gamemode.
#[get("/updates/<username>/<mode>")]
pub fn get_updates(db_pool: State<DbPool>, username: String, mode: u8) -> Result<Option<Json<Vec<Update>>>, String> {
    let db_conn = &*db_pool.get_conn();

    let usr: User = match get_user_from_username(db_conn, &username)? {
        Some(user) => user,
        None => { return Ok(None); },
    };

    // pull all updates belonging to the selected user from the database for the provided gamemode
    let updates = updates_dsl::updates
        .filter(updates_dsl::user_id.eq(usr.id))
        .filter(updates_dsl::mode.eq(mode as i16))
        .order(updates_dsl::update_time.asc())
        .load::<Update>(db_conn)
        .map_err(debug)?;

    Ok(Some(Json(updates)))
}

/// Returns all of a user's stored hsicores for a given gamemode.
#[get("/hiscores/<username>/<mode>")]
pub fn get_hiscores(db_pool: State<DbPool>, username: String, mode: u8) -> Result<Option<Json<Vec<Hiscore>>>, String> {
    let db_conn = &*db_pool.get_conn();

    let usr: User = match get_user_from_username(db_conn, &username)? {
        Some(user) => user,
        None => { return Ok(None); },
    };

    // pull all hiscores belonging to the selected user from the database for the provided gamemode
    let hiscores = hiscores_dsl::hiscores
        .filter(hiscores_dsl::user_id.eq(usr.id))
        .filter(hiscores_dsl::mode.eq(mode as i16))
        .order(hiscores_dsl::score_time.asc())
        .load::<Hiscore>(db_conn)
        .map_err(debug)?;

    Ok(Some(Json(hiscores)))
}

/// Returns the difference between a user's current stats and the last time their total PP score was different than its
/// current value.
#[get("/lastpp/<username>/<mode>")]
pub fn get_last_pp_diff(
    api_client: State<ApiClient>, db_pool: State<DbPool>, username: String, mode: u8
) -> Result<Option<Json<UpdateDiff>>, String> {
    let client = api_client.inner();
    let db_conn = &*db_pool.get_conn();

    let stats = client.get_stats(&username, mode)?;
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
            Ok(Some(Json(UpdateDiff::diff(last_different_update, &s, old_hiscores, cur_hiscores))))
        }
    }
}

/// Returns data for a set of beatmaps.  It first attempts to retrieve them from the database but if they aren't
/// stored, they will be retrieved from the osu! API and inserted.  Returns a Json-encoded hap of beatmap_id:beatmap
#[get("/beatmaps/<ids>/<mode>")]
pub fn get_beatmaps(
    api_client: State<ApiClient>, db_pool: State<DbPool>, ids: String, mode: u8
) -> Result<Option<Json<HashMap<i32, Beatmap>>>, String> {
    let ids: Vec<i32> = serde_json::from_str(&ids).map_err(debug)?;
    // TODO: Search the database and find all beatmaps that have IDs that are included in the parsed vector of ids.
    // TODO: Retrieve all beatmaps from the API (preferrably asynchronously) that are not contained in the database
    // TODO: Package up all results and return them
    unimplemented!();
}

/// Returns data for one beatmap.  It first attempts to retrieve the data from the database if it isn't found there
/// it is retrieved from the osu! API and inserted.
#[get("/beatmap/<id>/<mode>")]
pub fn get_beatmap(
    api_client: State<ApiClient>, db_pool: State<DbPool>, id: i32, mode: u8
) -> Result<Option<Json<Beatmap>>, String> {
    // TODO: Search the database for the beatmap with the supplied id
    // TODO: if not found in the database, return it from the API.
    unimplemented!();
}
