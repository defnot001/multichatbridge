CREATE TABLE IF NOT EXISTS users
(
    server_id   TEXT NOT NULL UNIQUE PRIMARY KEY,
    server_list TEXT NOT NULL,
    auth_token  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS configs
(
    client_id     TEXT NOT NULL UNIQUE PRIMARY KEY,
    subscriptions TEXT NOT NULL
);