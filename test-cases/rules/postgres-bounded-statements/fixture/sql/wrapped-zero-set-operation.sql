-- Built-in numeric wrappers preserve zero and avoid evaluating blocking inputs.
SELECT id FROM accounts EXCEPT SELECT account_id FROM orders LIMIT CAST(0 AS bigint);
SELECT id FROM accounts EXCEPT SELECT account_id FROM orders LIMIT ((+0));
SELECT id FROM accounts EXCEPT SELECT account_id FROM orders LIMIT -0;
SELECT id FROM accounts EXCEPT SELECT account_id FROM orders LIMIT CAST(0 AS int);
SELECT id FROM accounts EXCEPT SELECT account_id FROM orders LIMIT CAST(0 AS int2);
SELECT id FROM accounts EXCEPT SELECT account_id FROM orders LIMIT CAST(0 AS int4);
SELECT id FROM accounts EXCEPT SELECT account_id FROM orders LIMIT CAST(0 AS int8);
SELECT id FROM accounts EXCEPT SELECT account_id FROM orders LIMIT CAST(0 AS integer);
SELECT id FROM accounts EXCEPT SELECT account_id FROM orders LIMIT CAST(0 AS smallint);
SELECT id FROM accounts EXCEPT SELECT account_id FROM orders LIMIT CAST(0 AS numeric);
SELECT id FROM accounts EXCEPT SELECT account_id FROM orders LIMIT CAST(0 AS decimal);
SELECT id FROM accounts EXCEPT SELECT account_id FROM orders LIMIT CAST(0 AS dec);
SELECT id FROM accounts EXCEPT SELECT account_id FROM orders LIMIT CAST(0 AS real);
SELECT id FROM accounts EXCEPT SELECT account_id FROM orders LIMIT CAST(0 AS double precision);
SELECT id FROM accounts EXCEPT SELECT account_id FROM orders LIMIT CAST(0 AS float);
SELECT id FROM accounts EXCEPT SELECT account_id FROM orders LIMIT CAST(0 AS float4);
SELECT id FROM accounts EXCEPT SELECT account_id FROM orders LIMIT CAST(0 AS float8);
-- Custom types, nonzero counts, binds, strings and unknown operators prove no zero cap.
SELECT id FROM accounts WHERE id = $1 EXCEPT SELECT account_id FROM orders LIMIT CAST(0 AS public.custom_type);
SELECT id FROM accounts WHERE id = $1 EXCEPT SELECT account_id FROM orders LIMIT CAST(1 AS bigint);
SELECT id FROM accounts WHERE id = $1 EXCEPT SELECT account_id FROM orders LIMIT $1;
SELECT id FROM accounts WHERE id = $1 EXCEPT SELECT account_id FROM orders LIMIT '0';
SELECT id FROM accounts WHERE id = $1 EXCEPT SELECT account_id FROM orders LIMIT ~0;
SELECT id FROM accounts WHERE id = $1 EXCEPT SELECT account_id FROM orders LIMIT NULL;
SELECT id FROM accounts WHERE id = $1 EXCEPT SELECT account_id FROM orders LIMIT id;
