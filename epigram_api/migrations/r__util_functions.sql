CREATE OR REPLACE FUNCTION util.multibase_encode_hex(value BYTEA) 
  RETURNS TEXT
  AS $$
    SELECT 'f' || encode(value, 'hex')
  $$
  LANGUAGE SQL STABLE;
