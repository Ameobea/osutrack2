pub mod modes;

use std::io::Read;
use std::fmt::Debug;
use std::collections::HashMap;

use diesel::mysql::MysqlConnection;
use hyper;
use hyper::client::Response;
use r2d2::{ GetTimeout, Pool, PooledConnection, Config };
use r2d2_diesel_mysql::ConnectionManager;

use secret::DB_CREDENTIALS;

/// Utility function for making sure that a response is a 200 and then reading it into a String
pub fn process_response(mut res: Response) -> Result<String, String> {
    let _ = match res.status {
        hyper::NotFound => Err(String::from("Received error of 404 Not Found")),
        hyper::status::StatusCode::InternalServerError => {
            Err(String::from("Received error of 500 internal server error"))
        },
        hyper::Ok => Ok(()),
        _ => Err(format!("Received unknown error type: {:?}", res.status)),
    }?;

    let mut s = String::new();
    res.read_to_string(&mut s);

    Ok(s)
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
    let config = Config::default();
    let manager = ConnectionManager::<MysqlConnection>::new(format!("{}", DB_CREDENTIALS));
    Pool::new(config, manager).expect("Failed to create pool.")
}
