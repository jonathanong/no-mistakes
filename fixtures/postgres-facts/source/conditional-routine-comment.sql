-- Native IF parsing needs prepared metadata; signatures and Unicode positions survive.
DO $body$BEGIN
IF TRUE THEN
  COMMENT ON FUNCTION "Schéma"."Fun"(IN "Name" bigint, VARIADIC text[]) IS '雪';
  COMMENT ON FUNCTION no_signature IS 'COMMENT ON FUNCTION fake(integer) IS keywords';
  IF FALSE THEN
    COMMENT ON PROCEDURE p(integer) IS NULL;
  ELSE
    COMMENT ON FUNCTION empty_signature() IS 'empty';
  END IF;
END IF;
COMMENT ON FUNCTION direct(bigint) IS 'outside IF';
END$body$;
CREATE INDEX after_routine_comments ON neighbor(id);
