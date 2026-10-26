import { query } from "db";
// no-mistakes-disable-next-line postgres-no-transformed-uuid-predicates
query("SELECT 1 FROM accounts WHERE id::text = $1");
