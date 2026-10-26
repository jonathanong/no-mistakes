import { read, type QueryExecutor } from "@example/db";
import SQL from "sql-template-strings";
import { foreignTyped } from "./execution-ownership-helper.mjs";
declare const unknownBranch: boolean;

// The lexical typed sink and its replay describe one physical SQL occurrence.
function directOwner(query: QueryExecutor) {
  query("SELECT 1 OFFSET 101"); // EXPECT_OFFSET DIRECT_CLAUSE EXPECT_KNOWN_CALL
}
directOwner(read); // EXPECT_CALLER_PROOF DIRECT_REPLAY

// The lexical sink itself needs replay; a Direct-only owner index misses this overlap.
function recoveredOwner(query: QueryExecutor) {
  const statement = SQL`SELECT 1`;
  statement.append(SQL` OFFSET 102`); // EXPECT_OFFSET RECOVERED_CLAUSE
  query(statement); // EXPECT_KNOWN_CALL RECOVERED_OWNER_CALL
}
recoveredOwner(read); // EXPECT_CALLER_PROOF RECOVERED_REPLAY

foreignTyped(read); // EXPECT_CALLER_PROOF FOREIGN_REPLAY

// No configured lexical receiver exists here; the transport owns the sole proof.
function untypedOwner(query) {
  query("SELECT 1 OFFSET 104"); // EXPECT_OFFSET UNTYPED_CLAUSE
}
untypedOwner(read); // EXPECT_CALLER_PROOF UNTYPED_REPLAY

// One SQL token is executed at two distinct source call sites.
const saved = "SELECT 1 OFFSET 105"; // EXPECT_OFFSET_TWICE SHARED_CLAUSE
read(saved); // EXPECT_KNOWN_CALL SAVED_CALL_ONE
read(saved); // EXPECT_KNOWN_CALL SAVED_CALL_TWO

// Two distinct SQL tokens share one physical line and one executor call.
read("SELECT 1 OFFSET 106; SELECT 2 OFFSET 107"); // EXPECT_OFFSET_TWICE TWO_TOKENS EXPECT_KNOWN_CALL

function branchStatement(flag: boolean) {
  return flag ? "SELECT 1 OFFSET 108" : "SELECT 2 OFFSET 109"; // EXPECT_OFFSET_TWICE BRANCH_CLAUSES
}
read(branchStatement(unknownBranch)); // EXPECT_KNOWN_CALL BRANCH_CALL

// The lexical sink is Unknown; caller literals supply the known SQL alternatives.
function parameterizedOwner(query: QueryExecutor, statement: string) {
  query(statement); // EXPECT_UNKNOWN PARAMETERIZED_BODY
}
function parameterizedSql() {
  return "SELECT 1 OFFSET 110"; // EXPECT_OFFSET PARAMETERIZED_CLAUSE
}
// no-mistakes-disable-next-line postgres-no-offset
parameterizedOwner(read, parameterizedSql()); // EXPECT_DISABLED DISABLED_REPLAY
parameterizedOwner(read, parameterizedSql()); // EXPECT_CALLER_PROOF ACTIVE_REPLAY
