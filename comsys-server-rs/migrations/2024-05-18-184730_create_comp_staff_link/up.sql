-- Your SQL goes here
CREATE TABLE comp_staff_links (
    uid INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    cid INTEGER NOT NULL REFERENCES competitions(id) ON DELETE CASCADE,
    PRIMARY KEY (uid, cid)
)