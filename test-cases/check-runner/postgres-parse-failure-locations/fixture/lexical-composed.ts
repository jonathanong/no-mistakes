import { query } from "db";
const text = "SELECT 1; " +
  "SELECT 'unterminated";
query(text);
