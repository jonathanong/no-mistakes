import { query } from "@example/db";
query("SELECT 'ends\\', 'data' /* deadlock-safe: unique key */ FROM items WHERE id=ANY($1) FOR UPDATE");
