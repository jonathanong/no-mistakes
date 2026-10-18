import { query } from "@example/db";

// `tenant_id` may name the known orders table; the unknown peer blocks proof.
query("SELECT o.id FROM orders o JOIN other x ON x.id = o.id WHERE tenant_id = $1");
