CREATE TABLE users (
  id INT PRIMARY KEY NOT NULL,
  username VARCHAR(15) NOT NULL,
  first_update TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
  last_update TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP NOT NULL
);

ALTER TABLE `users` DEFAULT CHARSET=utf8mb4 COLLATE utf8mb4_unicode_ci;
