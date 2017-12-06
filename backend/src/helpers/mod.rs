pub mod modes;

use std::fmt::Debug;

use diesel::prelude::*;
use diesel::mysql::MysqlConnection;
use diesel::result::Error;
use reqwest::{self, Response, StatusCode};
use r2d2::Pool;
use r2d2_diesel::ConnectionManager;

use secret::DB_CREDENTIALS;
use models::{User, Update};

/// Utility function for making sure that a response is a 200 and then reading it into a String
pub fn process_response(mut res: Response) -> Result<String, String> {
    let _ = match res.status() {
        StatusCode::NotFound => Err(String::from("Received error of 404 Not Found")),
        StatusCode::InternalServerError => {
            Err(String::from("Received error of 500 internal server error"))
        },
        StatusCode::Ok => Ok(()),
        _ => Err(format!("Received unknown error type: {:?}", res.status())),
    }?;

    res.text().map_err(debug)
}

pub fn get_url(url: &str) -> Result<String, String> {
    process_response(
        reqwest::get(url).map_err(|err| format!("Error while sending request to osu! API: {:?}", err))?
    )
}

/// Given a type that can be debug-formatted, returns a String that contains its debug-formatted version.
pub fn debug<T>(x: T) -> String where T:Debug {
    format!("{:?}", x)
}

/// Attempts to convert the given &str into a T, panicing if it's not successful
pub fn parse_pair<T>(v: &str) -> T where T : ::std::str::FromStr {
    let res = v.parse::<T>();
    match res {
        Ok(val) => val,
        Err(_) => panic!(format!("Unable to convert given input into required type: {}", v)),
    }
}

pub const MYSQL_DATE_FORMAT: &'static str = "%Y-%m-%d %H:%M:%S";

pub fn create_db_pool() -> Pool<ConnectionManager<MysqlConnection>> {
    let manager = ConnectionManager::<MysqlConnection>::new(format!("{}", DB_CREDENTIALS));
    Pool::builder().build(manager).expect("Failed to create pool.")
}

/// Given a username, attempts to retrieve the stored `User` struct that goes along with it from the database.
pub fn get_user_from_username(connection: &MysqlConnection, username: &str) -> Result<Option<User>, String> {
    use schema::users::dsl as users_dsl;
    match users_dsl::users.filter(users_dsl::username.eq(username)).first(connection) {
        Ok(usr) => Ok(Some(usr)),
        Err(err) => match err {
            Error::NotFound => { return Ok(None); },
            _ => { return Err(format!("Error while getting user row from database: {:?}", err)); },
        }
    }
}

/// Finds the most recent update in the same game mode
pub fn get_last_update(user_id: i32, mode: u8, connection: &MysqlConnection) -> Result<Option<Update>, String> {
    use schema::updates::dsl as updates_dsl;

    let mut updates: Vec<Update> = updates_dsl::updates
        .filter(updates_dsl::user_id.eq(user_id))
        .filter(updates_dsl::mode.eq(mode as i16))
        .order(updates_dsl::update_time.desc())
        .limit(1)
        .load::<Update>(connection)
        .map_err(debug)?;

    if updates.len() == 0 { Ok(None) } else { Ok(Some(updates.drain(..).next().unwrap())) }
}
