import { query } from "@example/db";
// no-mistakes-disable-next-line postgres-lock-written-rows-only
query(`SELECT id FROM items FOR UPDATE`);
query(`LOCK TABLE items IN EXCLUSIVE MODE`); // no-mistakes-disable-line postgres-lock-written-rows-only
