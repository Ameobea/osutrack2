//! Maps the API endpoints to functions

use std::time::SystemTime;

use chrono::{DateTime, UTC};
use rocket_contrib::JSON;

use schema::{UpdateDiff, Stats};

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
