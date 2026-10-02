CREATE TABLE concurrently (very_long_column_name int CONSTRAINT keyword_constraint CHECK (very_long_column_name > 0));
CREATE INDEX CONCURRENTLY IF NOT EXISTS concurrently_index ON concurrently (very_long_column_name);
ALTER TABLE ONLY concurrently ADD COLUMN another_column int;
