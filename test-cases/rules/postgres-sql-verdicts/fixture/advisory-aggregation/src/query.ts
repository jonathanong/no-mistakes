import { query } from "@example/db";
declare const opaque: string;
declare const flag: boolean;
query(opaque);
query("SELECT FROM");
// The call and finite branch origins intentionally have different lines.
query(
  flag
    ? `CREATE TABLE sample (id int)`
    : `SELECT FROM`
);
// Same-line calls must retain separate aggregation identities.
query(opaque); query(opaque);
query(`CREATE TABLE prefix (id int); ${opaque}`);
