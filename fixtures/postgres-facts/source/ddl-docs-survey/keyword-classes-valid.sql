CREATE EVENT TRIGGER et ON ddl_command_start EXECUTE FUNCTION full();
CREATE EVENT TRIGGER et ON ddl_command_start EXECUTE FUNCTION public.select();
CREATE EVENT TRIGGER et ON ddl_command_start EXECUTE FUNCTION public.integer();
CREATE EVENT TRIGGER et ON ddl_command_start EXECUTE FUNCTION "integer"();
CREATE AGGREGATE full(integer) (SFUNC = int4pl, STYPE = integer);
CREATE CAST (text AS full) WITH INOUT;
DROP AGGREGATE IF EXISTS my_count(*);
DROP AGGREGATE full(integer) RESTRICT;
