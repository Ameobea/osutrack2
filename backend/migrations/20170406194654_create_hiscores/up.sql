CREATE TABLE hiscores (
  id INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  user_id INT NOT NULL,
  mode SMALLINT NOT NULL,
  beatmap_id INT NOT NULL,
  score INT NOT NULL,
  pp FLOAT NOT NULL,
  enabled_mods INT NOT NULL,
  rank VARCHAR(2) NOT NULL,
  score_time TIMESTAMP DEFAULT 0 NOT NULL,
  time_recorded TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX user ON hiscores (user_id);

CREATE INDEX pp ON hiscores(pp);
