CREATE TABLE user_orgs
(
    /*id SERIAL PRIMARY KEY,*/
    uid INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    oid INTEGER NOT NULL REFERENCES organisations(id) ON DELETE CASCADE,
    perm TEXT NOT NULL,
    PRIMARY KEY (uid, oid, perm)
)