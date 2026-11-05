import { query } from "@example/db";
query("SELECT E'ends\\'/* deadlock-safe: stored text */' FROM items WHERE id=ANY($1) FOR UPDATE");
