CREATE TABLE updates (
  id INT AUTO_INCREMENT PRIMARY KEY,
  user_id INT,
  mode TINYINT,
  count300 INT,
  count100 INT,
  count50 INT,
  playcount INT,
  ranked_score BIGINT,
  total_score BIGINT,
  pp_rank INT,
  level FLOAT,
  pp_raw FLOAT,
  accuracy FLOAT,
  count_rank_ss INT,
  count_rank_s INT,
  count_rank_a INT,
  update_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);