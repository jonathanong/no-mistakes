-- Malformed top-level statements remain failures on both sides of routine bodies.
ALTER TABLE before_body ADD COLUMN;
DO $body$
BEGIN
  FROBULATE do_body;
  ALTER TABLE body_table ADD COLUMN;
  CREATE TABLE preserved_body_table (id integer);
  EXECUTE 'FROBULATE static_body';
END
$body$;
CREATE FUNCTION unknown_function() RETURNS void LANGUAGE plpgsql AS $body$
BEGIN
  FROBULATE function_body;
END
$body$;
CREATE PROCEDURE unknown_procedure() LANGUAGE plpgsql AS $body$
BEGIN
  FROBULATE procedure_body;
END
$body$;
DO 'BEGIN FROBULATE quoted_body; END';
DO $outer$
BEGIN
  DO $inner$ BEGIN FROBULATE nested_body; END $inner$;
END
$outer$;
-- A lexer failure inside an identified body is uncertain, too.
DO $broken$
BEGIN
  value_text := 'unterminated;
END
$broken$;
ALTER TABLE after_body ADD COLUMN;
-- Parsing the body cannot repair malformed top-level routine grammar.
CREATE FUNCTION broken_header() RETURNS void LANGUAGE plpgsql BOGUS AS $$ BEGIN NULL; END $$;
