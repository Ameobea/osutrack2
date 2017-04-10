//! Functions or interfacing with the osu! API

use std::collections::HashMap;

use chrono::NaiveDateTime;
use diesel;
use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use hyper::client::Client;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use r2d2::Pool;
use r2d2_diesel_mysql::ConnectionManager;
use serde_json;

use secret::API_KEY;
use models::{Beatmap, NewUpdate, NewHiscore};
use schema;
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
    pub count300: String,
    pub count100: String,
    pub count50: String,
    pub playcount: String,
    pub ranked_score: String,
    pub total_score: String,
    pub pp_rank: String,
    pub level: String,
    pub pp_raw: String,
    pub accuracy: String,
    pub count_rank_ss: String,
    pub count_rank_s: String,
    pub count_rank_a: String,
    pub pp_country_rank: String,
    pub events: Vec<UpdateEvent>,
}

impl RawUpdate {
    /// Converts the raw representation into a representation suitable for storage in the database
    pub fn to_update(self, mode: u8) -> Result<NewUpdate, String> {
        Ok(NewUpdate {
            user_id: self.user_id.parse().map_err(debug)?,
            mode: mode as i16,
            count300: self.count300.parse().map_err(debug)?,
            count100: self.count100.parse().map_err(debug)?,
            count50: self.count50.parse().map_err(debug)?,
            playcount: self.playcount.parse().map_err(debug)?,
            ranked_score: self.ranked_score.parse().map_err(debug)?,
            total_score: self.total_score.parse().map_err(debug)?,
            pp_rank: self.pp_rank.parse().map_err(debug)?,
            level: self.level.parse().map_err(debug)?,
            pp_raw: self.pp_raw.parse().map_err(debug)?,
            accuracy: self.accuracy.parse().map_err(debug)?,
            count_rank_ss: self.count_rank_ss.parse().map_err(debug)?,
            count_rank_s: self.count_rank_s.parse().map_err(debug)?,
            count_rank_a: self.count_rank_a.parse().map_err(debug)?,
            pp_country_rank: self.pp_country_rank.parse().map_err(debug)?,
        })
    }
}

/// A raw list of user hiscores coming form the osu! API.  They quote their numbers so everything's a `String`.
#[derive(Clone, Deserialize)]
struct RawHiscore {
    pub beatmap_id: String,
    pub score: String,
    pub pp: String,
    pub mods: String,
    pub rank: String,
    pub score_time: String,
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
            mods: self.mods.parse().map_err(debug)?,
            rank: self.rank.parse().map_err(debug)?,
            score_time: NaiveDateTime::parse_from_str(&self.score_time, MYSQL_DATE_FORMAT).map_err(debug)?,
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
        Ok(Some(raw_updates[0].clone().to_update(mode)?))
    }

    pub fn get_user_best(&self, user_id: i32, mode: u8, count: u8) -> Result<Option<Vec<NewHiscore>>, String> {
        let request_url = format!("{}/get_user_best?k={}&u={}&mode={}&count={}", API_URL, API_KEY, user_id, mode, count);
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
        let results = Vec::with_capacity(raw_hiscores.len());
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
    // println!("{:?}", query);
    print_sql!(query);
    let conn: &MysqlConnection = &*client.pool.get().expect("Unable to get connection from pool");
    let res = query.execute(conn);
    // println!("{:?}", res);
}

/// Make sure that we're able to read values back out of the database
#[test]
fn test_beatmap_retrieve() {
    use schema::beatmaps::dsl::*;
    use models::Beatmap;

    let client = ApiClient::new();
    let conn: &MysqlConnection = &*client.pool.get().expect("Unable to get connection from pool");
    let results = beatmaps.filter(beatmap_id.eq(1031604))
        .load::<Beatmap>(conn);

    // println!("{:?}", results);
}

/// Make sure that we're able to retrieve user stats from the osu! API and parse them into a `NewUpdate`
#[test]
fn test_user_stats_fetch_store() {
    use helpers::modes::STANDARD;

    // get most recent user stats from the osu! API
    let client = ApiClient::new();
    let update = client.get_stats("ameo", STANDARD).unwrap().unwrap();
    println!("{:?}", update);

    // store the update into the database
    let conn: &MysqlConnection = &*client.pool.get().expect("Unable to get connection from pool");
    diesel::insert(&update).into(schema::updates::dsl::updates).execute(conn).unwrap();
}
