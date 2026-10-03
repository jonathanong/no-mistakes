-- Unquoted names fold to lowercase; quoted case, dots, and whitespace stay significant.
SELECT id FROM "Orders" ORDER BY id LIMIT $1;
SELECT id FROM Orders ORDER BY id LIMIT $1;
SELECT id FROM orders ORDER BY id LIMIT $1;
SELECT id FROM "work.Orders" ORDER BY id LIMIT $1;
SELECT id FROM " orders " ORDER BY id LIMIT $1;
