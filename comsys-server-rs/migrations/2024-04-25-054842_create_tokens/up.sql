
CREATE TABLE tokens
(
    id SERIAL PRIMARY KEY NOT NULL,
    hash TEXT NOT NULL,
    ttype INTEGER NOT NULL,
    owner INTEGER NOT NULL REFERENCES users(id),
    sub TEXT NOT NULL,
    UNIQUE (hash)
)