CREATE TABLE beatmaps (
  mode SMALLINT NOT NULL,
  beatmapset_id INT NOT NULL,
  beatmap_id INT NOT NULL PRIMARY KEY,
  approved SMALLINT NOT NULL,
  approved_date TIMESTAMP DEFAULT 0 NOT NULL,
  last_update TIMESTAMP DEFAULT 0 NOT NULL,
  total_length INT NOT NULL,
  hit_length INT NOT NULL,
  version VARCHAR(50) NOT NULL,
  artist VARCHAR(50) NOT NULL,
  title VARCHAR(50) NOT NULL,
  creator VARCHAR(50) NOT NULL,
  bpm FLOAT NOT NULL,
  source VARCHAR(50) NOT NULL,
  difficulty FLOAT NOT NULL,
  diff_size FLOAT NOT NULL,
  diff_overall FLOAT NOT NULL,
  diff_approach FLOAT NOT NULL,
  diff_drain FLOAT NOT NULL
);

ALTER TABLE `beatmaps` DEFAULT CHARSET=utf8mb4 COLLATE utf8mb4_bin;
