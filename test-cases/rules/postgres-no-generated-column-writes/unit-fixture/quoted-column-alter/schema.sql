CREATE TABLE items (id int, "Foo" int GENERATED ALWAYS AS (id) STORED);
-- Unquoted foo is a different column from quoted "Foo".
ALTER TABLE items ADD COLUMN foo int;
