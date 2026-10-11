import { query } from "@example/db";
query(flag
  ? "SELECT email FROM accounts"
  : "SELECT id FROM accounts");
