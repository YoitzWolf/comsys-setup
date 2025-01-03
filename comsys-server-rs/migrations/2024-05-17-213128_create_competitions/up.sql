-- Your SQL goes here

CREATE TABLE competitions (
    id SERIAL PRIMARY KEY NOT NULL,
    title TEXT NOT NULL,
    public BOOLEAN NOT NULL,
    organisation INTEGER NOT NULL REFERENCES organisations(id) ON DELETE CASCADE,
    start_date TIMESTAMP,
    ends_date TIMESTAMP,
    place TEXT,
    descr TEXT,
    scheme INTEGER NOT NULL,
    queues INTEGER NOT NULL,
    status INTEGER NOT NULL DEFAULT 0
)