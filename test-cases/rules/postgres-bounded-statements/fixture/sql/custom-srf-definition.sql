CREATE FUNCTION public.custom_srf(input uuid) RETURNS SETOF uuid
LANGUAGE sql AS $$ SELECT id FROM orders $$;
