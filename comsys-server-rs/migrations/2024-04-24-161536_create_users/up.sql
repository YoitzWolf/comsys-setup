CREATE TABLE
    users (
        id SERIAL PRIMARY KEY NOT NULL,
        login TEXT NOT NULL,
        selfname TEXT NOT NULL,
        hash TEXT NOT NULL,
        UNIQUE (id, login)
    )