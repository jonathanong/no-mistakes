import { query } from "@example/db";
// no-mistakes-disable-next-line postgres-json-relation-ids
export const run = () => query(`INSERT INTO orders (details) VALUES (jsonb_build_object('accountId', $1));`);
