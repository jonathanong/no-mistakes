import { query } from "@example/db";
declare const suffix: string;

// The known star projection violates policy even when a later clause is opaque.
query(`SELECT * FROM accounts ${suffix}`);
