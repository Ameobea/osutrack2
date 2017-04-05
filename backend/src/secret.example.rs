//! Private configuration settings example file.  Will be copied into the correct location by the build script.

pub struct DbCredentials {
    pub host: &'static str,
    pub username: &'static str,
    pub password: &'static str,
    pub database: &'static str,
}

pub const DB_CREDENTIALS: DbCredentials = DbCredentials {
    host: "localhost",
    username: "username",
    password: "password",
    database: "osutrack",
};
