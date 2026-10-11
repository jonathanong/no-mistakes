import { query, sql } from "@example/db";
let q;
if (flag) q = "SELECT 1"; else q = "SELECT 2";
query(q);
let prefix;
let suffix;
if (flag) { prefix = "SELECT "; suffix = "1"; }
else { prefix = "SELECT 2"; suffix = "0"; }
query(prefix + suffix); // SELECT 1 or SELECT 20, never SELECT 10 or SELECT 21
