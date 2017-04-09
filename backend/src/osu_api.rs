//! Functions or interfacing with the osu! API

use std::collections::HashMap;

use chrono::NaiveDateTime;
use diesel;
use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use hyper::client::Client;
use hyper::net::HttpsConnector;
use hyper_native_tls::NativeTlsClient;
use r2d2::{ GetTimeout, Pool, PooledConnection, Config };
use r2d2_diesel_mysql::ConnectionManager;
use serde_json::{self, Value};

use secret::API_KEY;
use models::NewBeatmap;
use schema;
use helpers::{debug, process_response, parse_pair, MYSQL_DATE_FORMAT, create_db_pool};

const API_URL: &'static str = "https://osu.ppy.sh/api";
const DATE_PARSE_ERROR: &'static str = "Unable to parse supplied datetime string into `NaiveDateTime`";

/// A client used to interface with the osu! API.
pub struct ApiClient {
    client: Client,
    pool: Pool<ConnectionManager<MysqlConnection>>,
}

impl ApiClient {
    fn new() -> ApiClient {
        let ssl = NativeTlsClient::new().unwrap();
        let connector = HttpsConnector::new(ssl);
        let client = Client::with_connector(connector);

        ApiClient {
            client: client,
            pool: create_db_pool(),
        }
    }

    /// Fetches beatmap metadata from the osu! API, automatically updating the internal betamap cache with the data.
    fn get_beatmap(&mut self, beatmap_id: usize, mode: u8) -> Result<NewBeatmap, String> {
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
            return Err(String::from("No beatmaps with that id were found!"))
        }
        let first = &raw[0];

        // parse the `HashMap` into a `NewBeatmap` manually since the values are provided as strings from the osu! API
        let beatmap = NewBeatmap {
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

        Ok(beatmap)
    }
}

/// Make sure we can run basic queries on the database using a connection pool
#[test]
fn basic_queries() {
    use diesel::connection::SimpleConnection;

    let mut client = ApiClient::new();
    let mut conn = &*client.pool.get().unwrap();
    let res = diesel::expression::dsl::sql::<::diesel::types::Bool>("SELECT 1")
        .get_result::<bool>(conn)
        .unwrap();
}

/// Try fetching a beatmap from the osu! API and make sure that it's parsed correctly.  Then insert it into the beatmap cache.
#[test]
fn test_beatmap_fetch_store() {
    use helpers::modes::STANDARD;
    let mut client = ApiClient::new();
    let beatmap = client.get_beatmap(1031604, STANDARD).unwrap();

    let query = diesel::insert(&beatmap)
        .into(schema::beatmaps::dsl::beatmaps);
    println!("{:?}", query);
    print_sql!(query);
    let conn: &MysqlConnection = &*client.pool.get().expect("Unable to get connection from pool");
    let res = query.execute(conn);
    println!("{:?}", res);
}

/// Make sure that we're able to read values back out of the database
#[test]
fn test_beatmap_retrieve() {
    use schema::beatmaps::dsl::*;
    use models::Beatmap;

    let mut client = ApiClient::new();
    let conn: &MysqlConnection = &*client.pool.get().expect("Unable to get connection from pool");
    let results = beatmaps.filter(beatmap_id.eq(1031604))
        .load::<Beatmap>(conn);

    println!("{:?}", results);
}
