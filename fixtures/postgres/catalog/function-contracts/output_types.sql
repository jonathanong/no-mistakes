CREATE FUNCTION f(IN value numeric(10, 2), OUT total numeric(10, 2), INOUT tags text[]) RETURNS record LANGUAGE sql AS $$SELECT 1$$
