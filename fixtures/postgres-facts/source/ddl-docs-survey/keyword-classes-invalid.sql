CREATE EVENT TRIGGER et ON ddl_command_start EXECUTE FUNCTION select();
CREATE EVENT TRIGGER et ON ddl_command_start EXECUTE FUNCTION integer();
CREATE ACCESS METHOD full TYPE INDEX HANDLER my_handler;
CREATE AGGREGATE integer(integer) (SFUNC = int4pl, STYPE = integer);
DROP AGGREGATE my_sum;
DROP AGGREGATE my_sum();
DROP AGGREGATE my_sum(*, integer);
DROP AGGREGATE my_sum(integer,);
