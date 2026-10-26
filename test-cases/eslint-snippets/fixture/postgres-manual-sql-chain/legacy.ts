import { query } from "@example/db";
// Released one-hop behavior intentionally remains broader than new chain proof.
let direct = "BEGIN";
query(direct);
const outer = "COMMIT";
function existing() { query(outer); }
query(sql`ROLLBACK`);
