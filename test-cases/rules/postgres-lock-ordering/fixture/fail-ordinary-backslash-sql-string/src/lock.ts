import { query } from "@example/db";
query("SELECT 'ends\\', '/* deadlock-safe: data */' FROM items WHERE id=ANY($1) FOR UPDATE");
