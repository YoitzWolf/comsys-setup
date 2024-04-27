-- Your SQL goes here

CREATE TABLE organisations
(
    id SERIAL PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    owner INTEGER NOT NULL REFERENCES users(id),
    UNIQUE (id, name)
)