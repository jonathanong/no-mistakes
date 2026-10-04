-- Semantic-zero pages must not become keyset sweeps; literal facts remain unchanged.
SELECT * FROM accounts ORDER BY id LIMIT 0::bigint;
SELECT * FROM accounts ORDER BY id LIMIT +0;
SELECT * FROM accounts ORDER BY id LIMIT (+(0::integer));
SELECT * FROM accounts ORDER BY id LIMIT -0;
SELECT * FROM accounts ORDER BY id FETCH FIRST 0 ROWS ONLY;
SELECT * FROM accounts ORDER BY id LIMIT 0.0;
SELECT * FROM accounts ORDER BY id LIMIT 0e0;
SELECT * FROM accounts ORDER BY id LIMIT 0::int;
SELECT * FROM accounts ORDER BY id LIMIT 0::int2;
SELECT * FROM accounts ORDER BY id LIMIT 0::int4;
SELECT * FROM accounts ORDER BY id LIMIT 0::int8;
SELECT * FROM accounts ORDER BY id LIMIT 0::smallint;
SELECT * FROM accounts ORDER BY id LIMIT 0::numeric(5,2);
SELECT * FROM accounts ORDER BY id LIMIT 0::decimal;
SELECT * FROM accounts ORDER BY id LIMIT 0::dec;
SELECT * FROM accounts ORDER BY id LIMIT 0::real;
SELECT * FROM accounts ORDER BY id LIMIT 0::double precision;
SELECT * FROM accounts ORDER BY id LIMIT 0::float;
SELECT * FROM accounts ORDER BY id LIMIT 0::float4;
SELECT * FROM accounts ORDER BY id LIMIT 0::float8;
-- Positive and unknown counts remain pages; a user-defined cast is not transparent.
SELECT * FROM accounts ORDER BY id LIMIT 1::bigint;
SELECT * FROM accounts ORDER BY id LIMIT +1;
SELECT * FROM accounts ORDER BY id LIMIT 0::public.custom_count;
SELECT * FROM accounts ORDER BY id LIMIT $1;
-- no-mistakes-disable-next-line postgres-sql-shape-policy
SELECT * FROM accounts ORDER BY id LIMIT +1;
SELECT * FROM accounts ORDER BY id LIMIT 0;
SELECT * FROM accounts ORDER BY id LIMIT NULL;
SELECT * FROM accounts ORDER BY id LIMIT 1e-1000;
