// no-mistakes-disable-file postgres-json-relation-ids
import { query } from "@example/db";
export const run = () => query(`INSERT INTO orders (details) VALUES (jsonb_build_object('accountId', $1));`);
