import { query } from "@example/db";
// The unknown relation is nested inside a derived table.
query("SELECT subq.id FROM (SELECT id FROM missing) AS subq");
