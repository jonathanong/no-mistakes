import sql, { type SQLStatement } from "sql-template-strings";
import { query } from "@example/db";
const reset = runtimeFlag;
let statement = reset ? "SELECT 1 OFFSET 9" : "SELECT 2";
try {
  mayThrow();
  statement = reset ? "SELECT 3" : "SELECT 4";
  query(statement); // finite
} catch {
  query(statement); // opaque
  query(reset ? statement : "SELECT 5"); // opaque
  statement = reset ? "SELECT 6" : "SELECT 7";
  query(statement); // finite
} finally {
  query(statement); // opaque
  const local = reset ? "SELECT 8" : "SELECT 9";
  query(local); // finite
}
query(statement); // opaque
/* Appending a safety clause after a throw does not make the catch value safe. */
const appended = sql`SELECT id FROM users`;
try {
  mayThrow();
  appended.append(sql` LIMIT 1`);
} catch {
  query(appended); // opaque prefix
  query(reset ? appended : sql`SELECT 10`); // opaque
}
query(appended); // opaque prefix
const finalized = sql`SELECT id FROM accounts`;
try {
  mayThrow();
  finalized.append(sql` WHERE id = 1`);
} finally {
  query(finalized); // opaque prefix
  query(reset ? finalized : sql`SELECT 11`); // opaque
}
/* Nested handlers discard the inner try's successful end snapshot too. */
let nested = reset ? "SELECT 12 OFFSET 4" : "SELECT 13";
try {
  mayThrow();
} catch {
  try {
    mayThrow();
    nested = reset ? "SELECT 14" : "SELECT 15";
  } catch {
    query(nested); // opaque
    const recovered = reset ? "SELECT 16" : "SELECT 17";
    query(recovered); // finite
  }
}
const stable = "SELECT 18";
try {
  mayThrow();
} catch (error) {
  query(stable); // unchanged
  query("SELECT 19"); // inline
}
query(stable); // unchanged
function localException() {
  let local = reset ? "SELECT 20 OFFSET 5" : "SELECT 21";
  try {
    mayThrow();
    local = reset ? "SELECT 22" : "SELECT 23";
  } catch {
    query(local); // opaque
  }
}
const escaped = sql`SELECT id FROM escaped_users`;
function capturedException() {
  try {
    unknownHelper(escaped);
  } catch {
    query(escaped); // opaque prefix
  }
}
query(reset ? escaped : sql`SELECT 24`); // finite
/* Each escape is an effect even after an earlier one cleared the snapshots. */
const repeated = sql`SELECT 25`;
const repeatedAlias = repeated;
external(repeatedAlias);
query(repeated); // opaque legacy builder
try {
  external(repeated);
} catch {
  query(repeated); // opaque prefix
  query(repeatedAlias); // opaque
}
function mutateAgain(input: SQLStatement): SQLStatement {
  input.append(sql` OFFSET 7`);
  return input;
}
const typed = sql`SELECT 26`;
mutateAgain(typed);
query(typed); // opaque prefix
try {
  mutateAgain(typed);
} catch {
  query(typed); // opaque prefix
}
external(stable);
try {
  external(stable);
} catch {
  query(stable); // unchanged
}
