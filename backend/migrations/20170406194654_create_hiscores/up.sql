CREATE TABLE hiscores (
  id INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  user_id INT,
  mode SMALLINT,
  beatmap_id INT,
  score INT,
  pp FLOAT,
  mods INT,
  rank INT,
  score_time TIMESTAMP DEFAULT 0,
  time_recorded TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX user ON hiscores (user_id);

CREATE INDEX pp ON hiscores(pp);
