-- Snowflake places UPDATE's FROM clause before SET; exercise sqlparser's alternate AST variant.
UPDATE accounts FROM orders SET accounts.name = 'x' WHERE FALSE;
