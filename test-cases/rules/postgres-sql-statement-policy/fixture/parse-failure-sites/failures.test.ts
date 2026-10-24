import { query } from '@example/db';
query(`SELECT 1;
-- Unsupported statements must retain separate starts below the executor call.
SELECT 1 FROM orders WHERE id::text COLLATE "C" = $1;
SELECT 1;
-- A valid sibling must not collapse either parse failure into the call origin.
SELECT 1 FROM orders WHERE id::text COLLATE "C" = $2;
`);
