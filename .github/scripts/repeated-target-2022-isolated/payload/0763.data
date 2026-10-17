COMMENT ON FUNCTION example_function() IS 'documentation';
ALTER INDEX example_parent ATTACH PARTITION example_child;
COMMENT ON TABLE example_table IS
  'First. '
  'Second.';
-- Quotes, modes, array types, and original Unicode spans matter.
COMMENT ON FUNCTION "Schéma"."Fun"(IN "Name" bigint, VARIADIC text[]) IS '雪''s '
 'documentation';
COMMENT ON PROCEDURE "Schéma".proc(integer) IS NULL;
COMMENT ON FUNCTION no_signature IS 'text';
ALTER INDEX "Schéma"."Parent" ATTACH PARTITION "Schéma"."Child";
ALTER INDEX IF EXISTS "Schéma"."Child" RENAME TO "Renamed";
COMMENT ON INDEX "Schéma"."Parent" IS 'parent';
COMMENT ON TABLE example_table IS 'a' -- continuation after a line comment
 'b'
 'c';
CREATE TABLE neighbor (label TEXT DEFAULT 'one'
 'two');
INSERT INTO neighbor (label) VALUES ('a'
 'b');
