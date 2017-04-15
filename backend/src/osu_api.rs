//! Functions or interfacing with the osu! API

use std::collections::HashMap;
use std::thread;

use chrono::NaiveDateTime;
use diesel;
use diesel::prelude::*;
use diesel::result::Error;
use diesel::mysql::MysqlConnection;
use hyper::client::Client;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use r2d2::Pool;
use r2d2_diesel_mysql::ConnectionManager;
use serde_json;

use secret::API_KEY;
use models::{Beatmap, NewUpdate, NewHiscore, User, NewUser};
use schema;
use schema::users::dsl as users_dsl;
use schema::updates::dsl as updates_dsl;
use schema::beatmaps::dsl as beatmaps_dsl;
use helpers::{debug, process_response, parse_pair, MYSQL_DATE_FORMAT, create_db_pool};

const API_URL: &'static str = "https://osu.ppy.sh/api";
const DATE_PARSE_ERROR: &'static str = "Unable to parse supplied datetime string into `NaiveDateTime`";

/// An event returned in a user stats response from the osu! API.  Since the API returns all its values as quoted by
/// default and really don't need to use these values right now, they stay as `String`s.
#[derive(Clone, Deserialize)]
struct UpdateEvent {
    pub display_html: String,
    pub beatmap_id: String,
    pub beatmapset_id: String,
    pub date: String,
    pub epicfactor: String,
}

/// A raw update coming from the osu! API, mapping directly to the JSON received from there.
#[derive(Clone, Deserialize)]
struct RawUpdate {
    pub user_id: String,
    pub username: String,
    pub count300: Option<String>,
    pub count100: Option<String>,
    pub count50: Option<String>,
    pub playcount: Option<String>,
    pub ranked_score: Option<String>,
    pub total_score: Option<String>,
    pub pp_rank: Option<String>,
    pub level: Option<String>,
    pub pp_raw: Option<String>,
    pub accuracy: Option<String>,
    pub count_rank_ss: Option<String>,
    pub count_rank_s: Option<String>,
    pub count_rank_a: Option<String>,
    pub pp_country_rank: Option<String>,
    pub events: Vec<UpdateEvent>,
}

impl RawUpdate {
    /// Converts the raw representation into a representation suitable for storage in the database.  If there are stats
    /// available for the user in the mode, will return `Ok(NewUpdate)`.  If the user exists but has no stats for the mode,
    /// returns `Err(None)`.  If some error occured during parsing/conversion, returns `Err(Some(String))`.
    pub fn to_update(self, mode: u8) -> Result<NewUpdate, Option<String>> {
        Ok(NewUpdate {
            user_id: self.user_id.parse().map_err(|err| Some(debug(err)) )?,
            mode: mode as i16,
            count300: self.count300.ok_or(None)?.parse().map_err(|err| Some(debug(err)) )?,
            count100: self.count100.ok_or(None)?.parse().map_err(|err| Some(debug(err)) )?,
            count50: self.count50.ok_or(None)?.parse().map_err(|err| Some(debug(err)) )?,
            playcount: self.playcount.ok_or(None)?.parse().map_err(|err| Some(debug(err)) )?,
            ranked_score: self.ranked_score.ok_or(None)?.parse().map_err(|err| Some(debug(err)) )?,
            total_score: self.total_score.ok_or(None)?.parse().map_err(|err| Some(debug(err)) )?,
            pp_rank: self.pp_rank.ok_or(None)?.parse().map_err(|err| Some(debug(err)) )?,
            level: self.level.ok_or(None)?.parse().map_err(|err| Some(debug(err)) )?,
            pp_raw: self.pp_raw.ok_or(None)?.parse().map_err(|err| Some(debug(err)) )?,
            accuracy: self.accuracy.ok_or(None)?.parse().map_err(|err| Some(debug(err)) )?,
            count_rank_ss: self.count_rank_ss.ok_or(None)?.parse().map_err(|err| Some(debug(err)) )?,
            count_rank_s: self.count_rank_s.ok_or(None)?.parse().map_err(|err| Some(debug(err)) )?,
            count_rank_a: self.count_rank_a.ok_or(None)?.parse().map_err(|err| Some(debug(err)) )?,
            pp_country_rank: self.pp_country_rank.ok_or(None)?.parse().map_err(|err| Some(debug(err)) )?,
        })
    }
}

/// A raw list of user hiscores coming form the osu! API.  They quote their numbers so everything's a `String`.
#[derive(Clone, Deserialize)]
struct RawHiscore {
    pub beatmap_id: String,
    pub score: String,
    pub pp: String,
    pub enabled_mods: String,
    pub rank: String,
    pub date: String,
}

impl RawHiscore {
    /// Converts the raw representation into a representation suitable for storage in the database
    pub fn to_new_hiscore(self, user_id: i32, mode: u8) -> Result<NewHiscore, String> {
        Ok(NewHiscore {
            user_id: user_id,
            mode: mode as i16,
            beatmap_id: self.beatmap_id.parse().map_err(debug)?,
            score: self.score.parse().map_err(debug)?,
            pp: self.pp.parse().map_err(debug)?,
            enabled_mods: self.enabled_mods.parse().map_err(debug)?,
            rank: self.rank,
            score_time: NaiveDateTime::parse_from_str(&self.date, MYSQL_DATE_FORMAT).map_err(debug)?,
        })
    }
}

/// A client used to interface with the osu! API.
pub struct ApiClient {
    client: Client,
    pool: Pool<ConnectionManager<MysqlConnection>>,
}

impl ApiClient {
    pub fn new() -> ApiClient {
        let ssl = NativeTlsClient::new().unwrap();
        let connector = HttpsConnector::new(ssl);
        let client = Client::with_connector(connector);

        ApiClient {
            client: client,
            pool: create_db_pool(),
        }
    }

    /// Fetches beatmap metadata from the osu! API, automatically updating the internal betamap cache with the data.
    pub fn get_beatmap(&self, beatmap_id: usize, mode: u8) -> Result<Option<Beatmap>, String> {
        let request_url = format!("{}/get_beatmaps?k={}&m={}&b={}", API_URL, API_KEY, mode, beatmap_id);
        let res = match self.client.get(&request_url).send() {
            Ok(res) => Ok(res),
            Err(err) => Err(format!("Error while sending request to osu! API: {:?}", err)),
        }?;

        // make sure that the response was what we expect it to be, a 200, and process it into a string
        let res_string: String = process_response(res)?;
        // try to parse the response into a vector of `String`:`String` `HashMap`s
        let raw: Vec<HashMap<String, String>> = serde_json::from_str(&res_string).map_err(debug)?;
        // make sure that we actually got a response
        if raw.len() == 0 {
            return Ok(None);
        }
        let first = &raw[0];

        // parse the `HashMap` into a `NewBeatmap` manually since the values are provided as strings from the osu! API
        let beatmap = Beatmap {
            mode: mode as i16,
            beatmapset_id: parse_pair(&first.get("beatmapset_id").unwrap()),
            beatmap_id: parse_pair(&first.get("beatmap_id").unwrap()),
            approved: parse_pair(&first.get("approved").unwrap()),
            approved_date: NaiveDateTime::parse_from_str(&first.get("approved_date").unwrap(), MYSQL_DATE_FORMAT)
                .expect(DATE_PARSE_ERROR),
            last_update: NaiveDateTime::parse_from_str(&first.get("approved_date").unwrap(), MYSQL_DATE_FORMAT)
                .expect(DATE_PARSE_ERROR),
            total_length: parse_pair(&first.get("total_length").unwrap()),
            hit_length: parse_pair(&first.get("hit_length").unwrap()),
            version: first.get("version").unwrap().clone(),
            artist: first.get("artist").unwrap().clone(),
            title: first.get("title").unwrap().clone(),
            creator: first.get("creator").unwrap().clone(),
            bpm: parse_pair(&first.get("bpm").unwrap()),
            source: first.get("source").unwrap().clone(),
            difficulty: parse_pair(&first.get("difficultyrating").unwrap()),
            diff_size: parse_pair(&first.get("diff_size").unwrap()),
            diff_overall: parse_pair(&first.get("diff_overall").unwrap()),
            diff_approach: parse_pair(&first.get("diff_approach").unwrap()),
            diff_drain: parse_pair(&first.get("diff_drain").unwrap()),
        };

        // insert the beatmap into the database in a separate thread
        let pool = self.pool.clone();
        let beatmap_clone = beatmap.clone();
        thread::spawn(move || {
            let conn: &MysqlConnection = &*pool.get().expect("Unable to get connection from pool");
            match diesel::insert(&beatmap_clone)
                .into(beatmaps_dsl::beatmaps)
                .execute(conn)
            {
                Ok(_) => (),
                Err(err) => println!("Error while attempting to insert beatmap into beatmap cache: {:?}", err),
            }
        });

        Ok(Some(beatmap))
    }

    /// Returns a user's current stats for a given gamemode.
    pub fn get_stats(&self, username: &str, mode: u8) -> Result<Option<NewUpdate>, String> {
        let request_url = format!("{}/get_user?k={}&u={}&m={}", API_URL, API_KEY, username, mode);
        let res = match self.client.get(&request_url).send() {
            Ok(res) => Ok(res),
            Err(err) => Err(format!("Error while sending request to osu! API: {:?}", err)),
        }?;

        // make sure that the response was what we expect it to be, a 200, and process it into a string
        let res_string: String = process_response(res)?;

        let raw_updates: Vec<RawUpdate> = serde_json::from_str(&res_string).map_err(debug)?;
        if raw_updates.len() == 0 {
            return Ok(None);
        }
        let raw_update = raw_updates[0].clone();
        let raw_clone = raw_update.clone();
        let parsed_update = raw_update.to_update(mode).map_err(|err_opt| -> String {
            match err_opt {
                Some(s) => s,
                None => format!("No stats available for user {} in that mode.", username),
            }
        })?;

        // in another thread, check if the user is in the database already.  If they are, make sure that their userid
        // and username match, updating them if they aren't.  If they're not in the db, add them.
        let pool = self.pool.clone();
        let parsed_clone = parsed_update.clone();
        thread::spawn(move || {
            let conn: &MysqlConnection = &*pool.get().map_err(debug).expect("Unable to get connection from pool in thread!");
            let user_id: i32 = raw_clone.user_id.parse().expect("Unable to parse user_id from string to i32");
            match users_dsl::users.find(user_id).first(conn) {
                Ok(usr) => {
                    // a user row exists for this user id, so check that the usernames match
                    let usr: User = usr;
                    diesel::update(users_dsl::users.find(usr.id))
                        .set(users_dsl::username.eq(&raw_clone.username))
                        .execute(conn)
                        .expect("Error while updating username");
                },
                Err(err) => {
                    // no user row exists, so insert one.
                    match err {
                        Error::NotFound => {
                            let usr = NewUser {
                                id: user_id,
                                username: raw_clone.username,
                            };

                            diesel::insert(&usr)
                                .into(users_dsl::users)
                                .execute(conn)
                                .expect("Unable to insert new user row into database.");
                        },
                        _ => println!("Unexpected error occured when searching database for username: {:?}", err),
                    }

                    // This is the first update for that user, so store this one
                    diesel::insert(&parsed_clone)
                        .into(updates_dsl::updates)
                        .execute(conn)
                        .map_err(debug)
                        .expect("Error while inserting first update into database");
                },
            }
        });

        Ok(Some(parsed_update))
    }

    pub fn get_user_best(&self, user_id: i32, mode: u8, count: u8) -> Result<Option<Vec<NewHiscore>>, String> {
        let request_url = format!("{}/get_user_best?k={}&u={}&m={}&limit={}", API_URL, API_KEY, user_id, mode, count);
        let res = match self.client.get(&request_url).send() {
            Ok(res) => Ok(res),
            Err(err) => Err(format!("Error while sending request to osu! API: {:?}", err)),
        }?;

        // make sure that the response was what we expect it to be, a 200, and process it into a string
        let res_string: String = process_response(res)?;

        let raw_hiscores: Vec<RawHiscore> = serde_json::from_str(&res_string).map_err(debug)?;
        if raw_hiscores.len() == 0 {
            return Ok(None)
        }

        // map all of the `RawHiscore`s into `NewHiscore`s
        let mut results = Vec::with_capacity(raw_hiscores.len());
        for raw_hiscore in raw_hiscores {
            let new_hiscore = raw_hiscore.to_new_hiscore(user_id, mode)?;
            results.push(new_hiscore);
        }

        Ok(Some(results))
    }
}

/// Make sure we can run basic queries on the database using a connection pool
#[test]
fn basic_queries() {
    let client = ApiClient::new();
    let conn = &*client.pool.get().unwrap();
    diesel::expression::dsl::sql::<::diesel::types::Bool>("SELECT 1")
        .get_result::<bool>(conn)
        .unwrap();
}

/// Try fetching a beatmap from the osu! API and make sure that it's parsed correctly.  Then insert it into the beatmap cache.
#[test]
fn test_beatmap_fetch_store() {
    use helpers::modes::STANDARD;
    let client = ApiClient::new();
    let beatmap = client.get_beatmap(1031604, STANDARD).unwrap().unwrap();

    let query = diesel::insert(&beatmap)
        .into(schema::beatmaps::dsl::beatmaps);
    let conn: &MysqlConnection = &*client.pool.get().expect("Unable to get connection from pool");
    query.execute(conn).unwrap();
}

/// Make sure that we're able to read values back out of the database
#[test]
fn test_beatmap_retrieve() {
    use schema::beatmaps::dsl::*;
    use models::Beatmap;

    let client = ApiClient::new();
    let conn: &MysqlConnection = &*client.pool.get().expect("Unable to get connection from pool");
    beatmaps.filter(beatmap_id.eq(1031604))
        .load::<Beatmap>(conn)
        .unwrap();
}

/// Make sure that we're able to retrieve user stats from the osu! API and parse them into a `NewUpdate`
#[test]
fn test_user_stats_fetch_store() {
    use helpers::modes::STANDARD;

    // get most recent user stats from the osu! API
    let client = ApiClient::new();
    let update = client.get_stats("ameo", STANDARD).unwrap().unwrap();

    // store the update into the database
    let conn: &MysqlConnection = &*client.pool.get().expect("Unable to get connection from pool");
    diesel::insert(&update).into(schema::updates::dsl::updates).execute(conn).unwrap();
}
