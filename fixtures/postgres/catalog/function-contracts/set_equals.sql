CREATE FUNCTION f() RETURNS int LANGUAGE sql SET search_path = public SET application_name = worker COST 2 SUPPORT schema.estimate AS $$SELECT 1$$
