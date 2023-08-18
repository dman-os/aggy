CREATE SCHEMA IF NOT EXISTS 
  posts;

COMMENT ON
  SCHEMA posts IS 'Issa self documenting, don''t you think?';

CALL util.apply_default_schema_config('posts');

CREATE TABLE posts.posts (
    created_at      TIMESTAMPTZ         NOT NULL    DEFAULT CURRENT_TIMESTAMP
,   updated_at      TIMESTAMPTZ         NOT NULL    DEFAULT CURRENT_TIMESTAMP


,   id            UUID                  NOT NULL  DEFAULT uuid_generate_v7()
,   author_id     UUID                  NOT NULL
,   epigram_id    BYTEA                 NOT NULL
,   title         TEXT                  NOT NULL
,   url           TEXT
,   body          TEXT

,   PRIMARY KEY(id)
,   FOREIGN KEY(author_id) REFERENCES auth.users
);

CALL util.apply_default_table_config('posts', 'posts');

CALL util.create_deleted_rows_table('posts', 'posts');
