CREATE FUNCTION "odd""(name"(OUT value integer, INOUT label text) RETURNS record LANGUAGE sql AS $$SELECT 1, 'a'$$
