import { query } from "@example/db";
query(flag
  ? "SELECT * FROM accounts"
  : "SELECT id FROM accounts");
