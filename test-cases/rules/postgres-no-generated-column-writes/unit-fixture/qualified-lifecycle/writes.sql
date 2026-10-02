UPDATE public.orders SET generated = 1;
UPDATE audit.orders SET generated = 1;
INSERT INTO public.orders VALUES (1, 2);
INSERT INTO audit.orders VALUES (1, 2);
UPDATE orders SET generated = 1;
