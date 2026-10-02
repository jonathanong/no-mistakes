UPDATE public.orders SET computed = 1;
UPDATE public.removed SET computed = 1;
UPDATE public."odd.name" SET computed = 1;
UPDATE public.ambiguous SET computed = 1;
UPDATE audit.ambiguous SET computed = 1;
UPDATE public.ambiguous SET retained = 1;
UPDATE shadowed SET computed = 1;
UPDATE public.shadowed SET computed = 1;
