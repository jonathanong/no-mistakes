-- A qualified END component is an object label, not a procedural terminator.
COMMENT ON COLUMN target.end IS 'column label';
COMMENT ON TABLE schema.end IS 'table label';
COMMENT ON FUNCTION schema.end() IS 'routine label';
CREATE FUNCTION qualified_end_body() RETURNS void BEGIN ATOMIC
  COMMENT ON COLUMN target. end IS 'spaced column label';
  COMMENT ON TABLE schema.end IS 'nested table label';
END;
DO $$BEGIN IF TRUE THEN
  COMMENT ON COLUMN target.end IS 'conditional column label';
END IF; END$$;
-- A real END still closes malformed metadata without its child semicolon.
CREATE FUNCTION malformed_end_body() RETURNS void BEGIN ATOMIC
  COMMENT ON COLUMN target.end IS 'missing delimiter'
END;
SELECT 99;
