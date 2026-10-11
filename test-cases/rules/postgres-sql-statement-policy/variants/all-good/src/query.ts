import { query } from "@example/db";
query(flag
  ? "SELECT 2"
  : "SELECT 1");
