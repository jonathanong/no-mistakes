import { query } from "@example/db";
declare const suffix: string;

// The opaque predicate value cannot remove the complete OFFSET clause.
query(`SELECT id FROM accounts WHERE id = ${suffix} OFFSET 2`);
