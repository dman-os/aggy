CREATE TABLE events (
    recieved_at      TIMESTAMPTZ         NOT NULL    DEFAULT CURRENT_TIMESTAMP

-- ,   row_id                  UUID  DEFAULT AS uuid_generate_v7
,   id                      BYTEA                       NOT NULL
,   pubkey                  BYTEA                       NOT NULL
,   created_at              TIMESTAMPTZ                 NOT NULL
,   kind                    INT                         NOT NULL
,   content                 TEXT                        NOT NULL
,   sig                     BYTEA                       NOT NULL
,   tags                    JSONB                       NOT NULL

,   PRIMARY KEY(id)
-- ,   UNIQUE(id)
);

CREATE INDEX ON
    events (pubkey);

CREATE INDEX ON
    events (kind);

CREATE INDEX ON
    events (created_at);

-- FIXME: better indices
CREATE INDEX ON
    events 
    USING GIN (tags);
