SELECT * FROM a LEFT JOIN b ON b.link = a.id LEFT JOIN c ON c.link = a.id WHERE num_nonnulls(COALESCE(a.id, b.id), c.id) = 1;
SELECT * FROM a LEFT JOIN b ON b.link = a.id LEFT JOIN c ON c.link = a.id WHERE num_nonnulls(b.id, c.id) = 1;
SELECT * FROM a LEFT JOIN b ON b.link = a.id LEFT JOIN c ON c.link = a.id WHERE 1 = pg_catalog.num_nonnulls(COALESCE(b.id, a.id), c.id);
SELECT * FROM public.a LEFT JOIN public.b ON b.link = a.id LEFT JOIN public.c ON c.link = a.id WHERE num_nonnulls(COALESCE(a.id, b.id), c.id) = 1;
