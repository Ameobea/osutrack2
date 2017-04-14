CREATE TABLE updates (
  id INT NOT NULL AUTO_INCREMENT PRIMARY KEY,
  user_id INT NOT NULL,
  mode SMALLINT NOT NULL,
  count300 INT NOT NULL,
  count100 INT NOT NULL,
  count50 INT NOT NULL,
  playcount INT NOT NULL,
  ranked_score BIGINT NOT NULL,
  total_score BIGINT NOT NULL,
  pp_rank INT NOT NULL,
  level FLOAT NOT NULL,
  pp_raw FLOAT NOT NULL,
  accuracy FLOAT NOT NULL,
  count_rank_ss INT NOT NULL,
  count_rank_s INT NOT NULL,
  count_rank_a INT NOT NULL,
  pp_country_rank INT NOT NULL,
  update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
);

CREATE INDEX user ON updates (user_id);
ALTER TABLE `updates` DEFAULT CHARSET=utf8mb4 COLLATE utf8mb4_unicode_ci;
