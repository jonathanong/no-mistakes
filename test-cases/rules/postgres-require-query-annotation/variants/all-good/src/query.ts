import { query } from "@example/db";
query(flag
  ? "/* items/read */ SELECT 2"
  : "/* items/read */ SELECT 1");
