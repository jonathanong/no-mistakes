import { query } from "@example/db";
let q;
if (flag) q = "SELECT 1"; else q = "SELECT 2";
q = query(q);
query(q); // A query result is opaque; the RHS call sees the previous binding.
