-- The native parser accepts bogus as text; prepared-token validation must reject it.
DO $$BEGIN IF TRUE THEN COMMENT ON TABLE t IS bogus; END IF; END$$;
CREATE INDEX valid_after_bad_comment ON t(id);
