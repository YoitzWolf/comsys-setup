CREATE TABLE user_orgs
(
    id SERIAL PRIMARY KEY,
    uid INTEGER NOT NULL REFERENCES users(id),
    oid INTEGER NOT NULL REFERENCES organisations(id),
    UNIQUE (uid, oid)
)