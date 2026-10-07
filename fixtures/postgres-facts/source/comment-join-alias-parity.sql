-- COMMENT is a relation/alias here, never a metadata statement.
SELECT * FROM users JOIN comment ON users.id = comment.id;
EXPLAIN (ANALYZE false) SELECT * FROM users JOIN comment ON users.id = comment.id;
SELECT * FROM users JOIN data AS comment ON users.id = comment.id;
PREPARE comment_join AS SELECT * FROM users JOIN data AS comment ON users.id = comment.id;
SELECT * FROM users JOIN rows() comment ON users.id = comment.id;
EXPLAIN SELECT * FROM users JOIN rows() comment ON users.id = comment.id;
SELECT * FROM users JOIN rows() AS comment ON users.id = comment.id;
EXPLAIN ANALYZE SELECT * FROM users JOIN rows() AS comment ON users.id = comment.id;
SELECT * FROM users JOIN begin comment ON users.id = comment.id;
EXPLAIN SELECT * FROM users JOIN begin comment ON users.id = comment.id;
DO $$BEGIN IF TRUE THEN
SELECT * FROM users JOIN comment ON users.id = comment.id;
END IF; END$$;
CREATE FUNCTION comment_alias_body() RETURNS integer LANGUAGE SQL BEGIN ATOMIC
SELECT * FROM users JOIN comment ON users.id = comment.id;
END;
SELECT 99;
