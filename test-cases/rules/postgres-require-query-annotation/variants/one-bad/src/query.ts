import { query } from "@example/db";
query(flag
  ? "SELECT 1"
  : "/* items/read */ SELECT 1");
