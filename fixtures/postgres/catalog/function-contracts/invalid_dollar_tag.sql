CREATE FUNCTION f() RETURNS int LANGUAGE sql AS $bad-tag$ SELECT 1 $bad-tag$
