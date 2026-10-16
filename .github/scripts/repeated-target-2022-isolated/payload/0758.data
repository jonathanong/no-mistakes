CREATE FUNCTION metadata_sql() RETURNS void BEGIN ATOMIC
  COMMENT ON FUNCTION schema."函"(IN amount numeric(12, 2), OUT text) IS U&'caf!00e9' UESCAPE '!';
  COMMENT ON TABLE target IS 'first '
    'second';
  COMMENT ON TABLE target IS NULL;
  SELECT 1;
END;
CREATE FUNCTION malformed_metadata() RETURNS void BEGIN ATOMIC
  COMMENT ON TABLE target IS bogus;
  SELECT 2;
END;
SELECT 62;
EXPLAIN ANALYZE COMMENT ON TABLE target IS 'not executable';
PREPARE bad AS COMMENT ON TABLE target IS 'not preparable';
EXPLAIN ANALYZE COMMENT ON TABLE target IS bogus;
DO $$ BEGIN IF true THEN
  COMMENT ON FUNCTION schema.fn() IS 'conditional';
  EXPLAIN COMMENT ON TABLE target IS 'invalid child';
END IF; END $$;
-- Missing child delimiter must not erase the function's authoritative END.
CREATE FUNCTION missing_comment_delimiter() RETURNS void BEGIN ATOMIC
  COMMENT ON TABLE target IS 'unfinished' END;
SELECT 63;
CREATE FUNCTION unfinished_metadata() RETURNS void BEGIN ATOMIC
  COMMENT ON TABLE target IS
