// no-mistakes-disable-file postgres-lock-written-rows-only
import { query } from "@example/db";
query(`SELECT id FROM items FOR UPDATE`);
query(`LOCK TABLE items IN EXCLUSIVE MODE`);
