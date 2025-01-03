-- Your SQL goes here

CREATE TABLE comp_data (
    id INTEGER PRIMARY KEY NOT NULL UNIQUE REFERENCES competitions(id) ON DELETE CASCADE,
    queues BYTEA NOT NULL
)