import sql, { SQLStatement } from "sql-template-strings";
import { query } from "@example/db";

function withOffset(input: SQLStatement): SQLStatement {
  return input.append(sql` OFFSET 7`);
}

const conditional = sql`SELECT id FROM conditional_users`;
query(withOffset(conditional) ? conditional : sql`SELECT 2`);
const logical = sql`SELECT id FROM logical_users`;
query(withOffset(logical) || sql`SELECT 3`);
const interpolated = sql`SELECT id FROM interpolated_users`;
query(sql`${withOffset(interpolated)} ${interpolated}`);
const nested = sql`SELECT id FROM nested_users`;
query(isReady(withOffset(nested)) ? nested : sql`SELECT 4`);
let assigned = sql`SELECT id FROM assigned_users`;
query((assigned += sql` LIMIT 1`) ? assigned : sql`SELECT 5`);
let updated = sql`SELECT id FROM updated_users`;
query(updated++ ? updated : sql`SELECT 6`);
const appended = sql`SELECT id FROM appended_users`;
query(appended.append(sql` LIMIT 1`) ? appended : sql`SELECT 7`);
const computed = sql`SELECT id FROM computed_users`;
query(computed["append"](sql` LIMIT 1`) ? computed : sql`SELECT 8`);

// Fresh inputs have no outer builder snapshot to invalidate.
query(withOffset(sql`SELECT id FROM fresh_users`) ? "SELECT 9" : "SELECT 10");
const sole = sql`SELECT id FROM sole_users`;
query(sql`${withOffset(sole)}`);
query(runtimeFlag ? "SELECT 11" : "SELECT 12");

let counter = 0;
query((counter = runtimeCount) ? "SELECT 13" : "SELECT 14");
query(counter++ ? "SELECT 15" : "SELECT 16");
const scalar = { value: 0 };
query((scalar.value = runtimeCount) ? "SELECT 17" : "SELECT 18");
query(scalar.value++ ? "SELECT 19" : "SELECT 20");
query(scalar.check() ? "SELECT 21" : "SELECT 22");

// Creating callbacks does not run their mutation bodies.
const deferredFunction = sql`SELECT id FROM deferred_function_users`;
query(function () { deferredFunction.append(sql` LIMIT 1`); } ? "SELECT 23" : "SELECT 24");
const deferredArrow = sql`SELECT id FROM deferred_arrow_users`;
query((() => deferredArrow.append(sql` LIMIT 1`)) ? "SELECT 25" : "SELECT 26");

// A branch can lose exact object identity while retaining SQL alternatives.
let branchBuilder = sql`SELECT id FROM original_branch_users`;
if (runtimeFlag) branchBuilder = sql`SELECT id FROM replacement_branch_users`;
query(branchBuilder.append(sql` LIMIT 1`) ? branchBuilder : sql`SELECT 27`);
let scalarChoice = runtimeFlag ? true : false;
query((scalarChoice = runtimeFlag) ? "SELECT 28" : "SELECT 29");
