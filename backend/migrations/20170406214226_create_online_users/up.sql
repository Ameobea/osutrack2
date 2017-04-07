CREATE TABLE online_users (
  time_recorded TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL PRIMARY KEY,
  users INT,
  operators INT,
  voiced INT
);
