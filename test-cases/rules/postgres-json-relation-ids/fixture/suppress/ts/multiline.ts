import { query } from "@example/db";
export const run = () => query(`
 INSERT INTO orders (details) VALUES (
  jsonb_build_object(
   -- no-mistakes-disable-next-line postgres-json-relation-ids
   'accountId', $1))`);
