CREATE SCHEMA IF NOT EXISTS 
  grams;

COMMENT ON
  SCHEMA grams IS 'The main thing';

CALL util.apply_default_schema_config('grams');

CREATE TABLE grams.grams (
    created_at      TIMESTAMPTZ         NOT NULL    DEFAULT CURRENT_TIMESTAMP
-- ,   updated_at      TIMESTAMPTZ         NOT NULL    DEFAULT CURRENT_TIMESTAMP


,   id                      BYTEA                       NOT NULL
,   content                 TEXT                        NOT NULL
,   mime                    TEXT                        NOT NULL
,   parent_id               BYTEA
,   sig                     BYTEA                       NOT NULL
,   author_pubkey           BYTEA                       NOT NULL
,   author_alias            TEXT
,   author_notif_email      extensions.CITEXT

    -- all constraints (besides not null) go after the columns
,   PRIMARY KEY(id)
-- ,   UNIQUE(email)
);

CALL util.apply_default_table_config('grams', 'grams');

CALL util.create_deleted_rows_table('grams', 'grams');

---

-- CREATE TABLE grams.authors (
--     created_at      TIMESTAMPTZ         NOT NULL    DEFAULT CURRENT_TIMESTAMP
-- ,   updated_at      TIMESTAMPTZ         NOT NULL    DEFAULT CURRENT_TIMESTAMP
-- 
-- 
-- ,   id               TEXT                        NOT NULL  DEFAULT uuid_generate_v7()
-- ,   pub_key          BYTEA                       NOT NULL
-- ,   alias            TEXT                        NOT NULL
-- ,   sig_line         BYTEA                       NOT NULL
-- ,   notif_email      extensions.CITEXT            NOT NULL
-- 
-- ,   PRIMARY KEY(id)
-- );
