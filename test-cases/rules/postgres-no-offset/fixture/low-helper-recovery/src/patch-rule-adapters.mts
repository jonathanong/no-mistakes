import { read } from "@example/db";
import SQL from "sql-template-strings";
declare const flag: boolean;

function unknownTable() {
  return "SELECT 1 FROM missing_table WHERE id::text = $1";
}
function unknownColumn() {
  return "SELECT 1 FROM accounts WHERE missing_column::text = $1";
}
function textColumn() {
  return "SELECT 1 FROM accounts WHERE name::text = $1";
}
read(unknownTable()); // UUID_UNKNOWN_TABLE
read(unknownColumn()); // UUID_UNKNOWN_COLUMN
read(textColumn()); // UUID_TEXT_CLEAR

function suppressedUuid() {
  return "SELECT 1 FROM accounts WHERE id::text = $1"; // SUPPRESSED_UUID_ORIGIN
}
// no-mistakes-disable-next-line postgres-no-transformed-uuid-predicates
read(suppressedUuid()); // SUPPRESSED_UUID_CALL

function mixedUuid(enabled: boolean) {
  if (enabled) return "SELECT 1 FROM accounts WHERE id::text = $1"; // UUID_MIX_ORIGIN
  return "SELECT (";
}
function mixedOffset(enabled: boolean) {
  if (enabled) return "SELECT 1 OFFSET 1"; // OFFSET_MIX_ORIGIN
  return "SELECT (";
}
read(mixedUuid(flag)); // UUID_MIX_CALL
read(mixedOffset(flag)); // OFFSET_MIX_CALL

function multilineOffset() {
  return SQL`SELECT 1
    OFFSET 2 -- MULTILINE_OFFSET_ORIGIN
  `;
}
read(multilineOffset()); // MULTILINE_OFFSET_CALL

// Bound values cannot become authored quoted text, even if the SQL parser accepts it.
function quotedBind(value: unknown) {
  return SQL`SELECT '${value}'`;
}
// This generated placeholder also exercises tokenizer failure, not only placement failure.
function brokenBind(value: unknown) {
  return SQL`SELECT '${value}`;
}
read(quotedBind(flag)); // INVALID_QUOTED_BIND
read(brokenBind(flag)); // INVALID_TOKENIZER_BIND
