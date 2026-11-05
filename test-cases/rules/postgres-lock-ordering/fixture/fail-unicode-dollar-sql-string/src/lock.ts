import { query } from "@example/db";
query("SELECT $\u03c3$ /* deadlock-safe: data */ $\u03c3$ FROM items WHERE id=ANY($1) FOR UPDATE");
