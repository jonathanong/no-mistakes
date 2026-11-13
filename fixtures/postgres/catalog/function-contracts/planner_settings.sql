CREATE FUNCTION f() RETURNS SETOF integer LANGUAGE sql SET search_path TO public, stable SET application_name FROM CURRENT SUPPORT public.estimate COST 3.5 ROWS 8 PARALLEL SAFE AS $$SELECT 1$$
