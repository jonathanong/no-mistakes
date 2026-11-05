import { read, sql } from "@example/db";

declare const options: { lower: boolean; hashes: boolean };
declare const unknown: number;

read(`SELECT id FROM orders WHERE id = $${options.lower ? 4 : 3}`);
read(`SELECT id FROM orders WHERE id = $${options.lower ? "4" : "3"}`);
const predicate = options.hashes
  ? ` AND id = ANY($${options.lower ? 4 : 3}::int[])`
  : "";
read(`SELECT id FROM orders WHERE account_id = $1${predicate}`);
read(`SELECT ${options.lower ? 0x10 : 1_000}`);
read(`SELECT ${options.lower ? 1e21 : 1e-7}`);
read(`SELECT ${(options.lower ? 0 : 4) || 3}`);
read(`SELECT ${(options.lower ? 0 : 4) && 3}`);
read(`SELECT ${options.lower ? 0 : unknown}`);
read(`SELECT ${(options.lower ? 1 : 2) + 3}`);
read(sql`SELECT id FROM orders WHERE id = ${options.lower ? 4 : 3}`);
