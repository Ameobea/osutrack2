CREATE TABLE users (
  osu_id INT PRIMARY KEY,
  username VARCHAR(15),
  first_update TIMESTAMP DEFAULT 0,
  last_update TIMESTAMP DEFAULT 0
);
