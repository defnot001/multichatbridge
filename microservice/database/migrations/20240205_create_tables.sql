CREATE TABLE IF NOT EXISTS users
(
    server_id   TEXT NOT NULL UNIQUE PRIMARY KEY,
    server_list TEXT NOT NULL,
    auth_token  TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS configs
(
    identifier    TEXT NOT NULL UNIQUE PRIMARY KEY,
    server_id     TEXT NOT NULL,
    client_id     TEXT NOT NULL UNIQUE,
    subscriptions TEXT NOT NULL
);