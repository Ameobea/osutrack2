CREATE TABLE online_users (
  time_recorded TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL PRIMARY KEY,
  users INT NOT NULL,
  operators INT NOT NULL,
  voiced INT NOT NULL
);

ALTER TABLE `online_users` DEFAULT CHARSET=utf8mb4 COLLATE utf8mb4_general_ci;
