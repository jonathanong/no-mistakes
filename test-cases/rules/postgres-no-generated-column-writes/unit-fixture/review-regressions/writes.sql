UPDATE public.orders SET updated_at = now();
UPDATE orders SET note = E'it\'s; pending', updated_at = now();
/* outer /* inner */ ; still outer */ UPDATE orders SET updated_at = now();
UPDATE computed_orders SET updated_at = 1;
INSERT INTO computed_orders VALUES (1, 2);
UPDATE orders SET note = 'it''s; pending', updated_at = now();
UPDATE orders SET note = E'continued\
line; pending', updated_at = now();
UPDATE public."dotted.orders" SET updated_at = now();
