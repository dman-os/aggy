CREATE SCHEMA IF NOT EXISTS 
  extensions;

CREATE EXTENSION IF NOT EXISTS 
    "uuid-ossp"
  WITH SCHEMA
    extensions;

CREATE EXTENSION IF NOT EXISTS 
    citext
  WITH SCHEMA
    extensions;

---

CREATE SCHEMA IF NOT EXISTS 
  util;

COMMENT ON
  SCHEMA util IS 'Helper utilities.';

---

CREATE OR REPLACE FUNCTION 
    util.maintain_updated_at()
  RETURNS TRIGGER AS 
  $body$
      BEGIN
          NEW.updated_at := CURRENT_TIMESTAMP;
          RETURN NEW;
      END;
  $body$ LANGUAGE PLpgSQL;

---
CREATE OR REPLACE PROCEDURE 
    util.apply_default_schema_config(
      schema_name TEXT
    )
  AS $body$
    BEGIN
      -- EXECUTE FORMAT('CREATE SCHEMA IF NOT EXISTS %I', schema_name);
      --
      -- EXECUTE FORMAT('GRANT USAGE ON SCHEMA %I TO postgres', schema_name);
      --
      -- EXECUTE FORMAT('ALTER DEFAULT PRIVILEGES 
      -- IN SCHEMA %I 
      -- GRANT ALL ON TABLES TO 
      -- postgres', schema_name);
      --
      -- EXECUTE FORMAT('ALTER DEFAULT PRIVILEGES 
      -- IN SCHEMA %I 
      -- GRANT ALL ON FUNCTIONS 
      -- TO postgres', schema_name);
      --
      -- EXECUTE FORMAT('ALTER DEFAULT PRIVILEGES 
      -- IN SCHEMA %I 
      -- GRANT ALL ON SEQUENCES 
      -- TO postgres', schema_name);
    END;
  $body$ LANGUAGE PLpgSQL;
COMMENT ON 
  PROCEDURE util.apply_default_schema_config 
  IS 'Default config to apply to schemas after creation';


CALL util.apply_default_schema_config('extensions');
CALL util.apply_default_schema_config('utils');

---

CREATE OR REPLACE PROCEDURE 
    util.apply_default_table_config(
      schema_name TEXT
      ,table_name TEXT
    )
  AS $body$
      BEGIN
        -- EXECUTE FORMAT('ALTER TABLE %I.%I OWNER to postgres', schema_name, table_name);
        EXECUTE FORMAT('
        CREATE OR REPLACE TRIGGER maintain_updated_at
        BEFORE UPDATE
        ON %I.%I
        FOR EACH ROW
        EXECUTE PROCEDURE util.maintain_updated_at()', schema_name, table_name);
        /* EXECUTE FORMAT(
            'ALTER TABLE IF EXISTS %I.%I ENABLE ROW LEVEL SECURITY;'
            ,schema_name
            ,table_name
        ); */
      END;
  $body$ LANGUAGE PLpgSQL;

COMMENT ON 
  PROCEDURE util.apply_default_table_config 
  IS 'Default configurations to apply to tables after creation
This assumes that the table has a `updated_at` column.';

---

CREATE OR REPLACE PROCEDURE 
    util.create_deleted_rows_table(
      schema_name TEXT
      ,table_name TEXT
    )
  AS $body$
      BEGIN
        EXECUTE FORMAT('
CREATE TABLE %I.%I_deleted
(
    deleted_at    TIMESTAMPTZ     NOT NULL    DEFAULT CURRENT_TIMESTAMP
,   row           JSONB           NOT NULL
);
          ', schema_name, table_name);
        -- EXECUTE FORMAT('ALTER TABLE %I.%I_deleted OWNER to postgres', schema_name, table_name);
        -- EXECUTE FORMAT(
        --     'ALTER TABLE IF EXISTS %I.%I_deleted ENABLE ROW LEVEL SECURITY;'
        --     ,schema_name
        --     ,table_name
        -- );
      END;
  $body$ LANGUAGE PLpgSQL;

COMMENT ON 
  PROCEDURE util.create_deleted_rows_table 
  IS 'Create a deleted rows store table under the specified names.';
