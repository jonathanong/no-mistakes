CREATE FUNCTION f(n integer DEFAULT (public.return_value(1))) RETURNS integer LANGUAGE sql RETURN n + 1
