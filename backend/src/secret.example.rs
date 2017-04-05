//! Private configuration settings example file.  Will be copied into the correct location by the build script.

pub struct DbCredentials {
    host: &'static str,
    username: &'static str,
    password: &'static str,
    database: &'static str,
}

pub const DB_CREDENTIALS: DbCredentials = DbCredentials {
    host: "localhost",
    username: "username",
    password: "password",
    database: "osutrack",
}
