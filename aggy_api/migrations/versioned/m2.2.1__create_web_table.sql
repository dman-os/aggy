CREATE SCHEMA IF NOT EXISTS 
  web;

COMMENT ON
  SCHEMA web IS 'Tables relating to the web app';

CALL util.apply_default_schema_config('web');

CREATE TABLE web.sessions (
    created_at      TIMESTAMPTZ         NOT NULL    DEFAULT CURRENT_TIMESTAMP
,   updated_at      TIMESTAMPTZ         NOT NULL    DEFAULT CURRENT_TIMESTAMP

,   id                UUID            NOT NULL      DEFAULT uuid_generate_v7()
,   user_id           UUID
,   expires_at        TIMESTAMPTZ     NOT NULL
,   ip_addr           INET            NOT NULL
,   user_agent        TEXT            NOT NULL

,   PRIMARY KEY(id)
,   FOREIGN KEY(user_id) REFERENCES auth.users
);

CALL util.apply_default_table_config('web', 'sessions');

CALL util.create_deleted_rows_table('web', 'sessions');
