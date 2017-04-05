CREATE TABLE users (
  id INT AUTO_INCREMENT PRIMARY KEY,
  osu_id INT,
  username VARCHAR(15),
  first_update TIMESTAMP DEFAULT 0,
  last_update TIMESTAMP DEFAULT 0
);
