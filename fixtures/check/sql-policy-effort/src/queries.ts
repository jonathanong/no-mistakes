import { query } from "@example/db";
import sql from "sql-template-strings";

query("SELECT id FROM accounts WHERE deleted_at IS NULL");
const oneLocal = "SELECT id FROM accounts WHERE deleted_at IS NULL";
query(oneLocal);
query(sql`SELECT id FROM accounts WHERE deleted_at IS NULL AND id = ${"account"}`);

query("SELECT * FROM accounts");
const oneBadLocal = "SELECT * FROM accounts";
query(oneBadLocal);

// Bounded helper recovery stays supported at low; it is not literal-only.
function buildGood() {
  return "SELECT id FROM accounts WHERE deleted_at IS NULL";
}
query(buildGood());
function buildBad() {
  return "SELECT * FROM accounts";
}
query(buildBad());

// Unknown SQL and invalid SQL are separate uncertainty obligations.
declare function opaqueSql(): string;
query(opaqueSql());
query("SELECT FROM");

// A suppressed SELECT star must stay hidden, including with a custom message.
// no-mistakes-disable-next-line postgres-explicit-columns
query("SELECT * FROM accounts WHERE deleted_at IS NULL");
