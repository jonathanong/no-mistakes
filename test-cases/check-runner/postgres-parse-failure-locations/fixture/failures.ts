import { query } from "db";
query(`SELECT 1;
-- Recovery starts below the JavaScript call line.
SELECT 1 FROM orders WHERE id::text COLLATE "C" = $1;
SELECT 1;
-- A second statement keeps its own physical origin.
SELECT 1 FROM orders WHERE id::text COLLATE "C" = $2;
`);
