import sql from "sql-template-strings";
import { query } from "@example/db";
const flag = runtimeFlag;
const choice = otherFlag;
const statement = sql`SELECT id FROM users`;
const alias = statement;
function runBuilder() {
  query(flag ? statement : sql`SELECT 1`); // captured opaque
  query(flag ? alias : sql`SELECT 2`); // captured opaque
}
statement.append(sql` OFFSET 1`);
runBuilder();
query(flag ? statement : sql`SELECT 3`); // finite
const conditionalBuilder = choice ? sql`SELECT 4` : sql`SELECT 5`;
function runConditionalBuilder() {
  query(flag ? conditionalBuilder : sql`SELECT 6`); // captured opaque
}
const primitive = "SELECT id FROM accounts";
function runPrimitive() {
  query(flag ? primitive : "SELECT 7"); // finite
}
const conditionalPrimitive = choice ? "SELECT 8" : "SELECT 9";
function runConditionalPrimitive() {
  query(flag ? conditionalPrimitive : "SELECT 10"); // finite three
}
let text = "SELECT id FROM text_users";
const copied = text;
function runMutableText() {
  query(flag ? text : "SELECT 11"); // captured opaque
  query(flag ? copied : "SELECT 12"); // finite
}
text = "SELECT id FROM text_users OFFSET 2";
runMutableText();
query(flag ? text : "SELECT 13"); // finite
var variable = "SELECT id FROM variable_users";
function runMutableVar() {
  query(flag ? variable : "SELECT 14"); // captured opaque
}
variable = "SELECT id FROM variable_users OFFSET 3";
runMutableVar();
function localBuilder() {
  const local = sql`SELECT id FROM local_users`;
  query(flag ? local : sql`SELECT 15`); // finite
  {
    const blockLocal = sql`SELECT id FROM block_users`;
    query(flag ? blockLocal : sql`SELECT 16`); // finite
    function nested() {
      query(flag ? blockLocal : sql`SELECT 17`); // captured opaque
    }
    blockLocal.append(sql` OFFSET 4`);
    nested();
    query(flag ? blockLocal : sql`SELECT 18`); // finite
  }
}
const legacy = sql`SELECT id FROM legacy_users`;
function bareLegacy() {
  query(legacy); // unchanged builder
}
query(legacy); // unchanged builder
