DO $$BEGIN IF TRUE THEN COMMENT ON FUNCTION f(bigint DEFAULT 1) IS 'invalid'; END IF; END$$;
CREATE INDEX after_invalid_signature ON neighbor(id);
DO $$BEGIN IF TRUE THEN COMMENT ON FUNCTION f() IS 'x' extra; END IF; END$$;
CREATE INDEX after_invalid_boundary ON neighbor(id);
