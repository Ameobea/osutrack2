//! This file contains a macro call to automatically generate code based on the schema of the database.  It should be
//! updated with your own database credentials and copied to `schema.rs` by the build script.

infer_schema!("mysql://username:password@host.com/dbname");
