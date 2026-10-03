import { query } from "@example/db";
query(`SELECT id FROM t
-- no-mistakes-disable-next-line postgres-no-offset
OFFSET 3`);
