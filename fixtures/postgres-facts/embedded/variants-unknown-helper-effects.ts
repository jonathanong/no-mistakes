import { sql } from "@example/db"; import { type SQLStatement } from "sql-template-strings"; /* Configured tags splice fragments; sql-template-strings binds them. */
import { query } from "@example/db";
const gate = runtimeFlag;
function choose(input: SQLStatement): boolean {
  if (runtimeFlag) input.append(sql` OFFSET 9`);
  return runtimeFlag;
}
const statement = sql`SELECT id FROM users`;
const alias = statement;
const unrelated = sql`SELECT id FROM accounts`;
query(choose(statement) ? statement : sql`SELECT 1`); // opaque
query(statement); // opaque known-prefix
query(alias); // opaque
query(gate ? unrelated : sql`SELECT 2`); // finite
query(unrelated); // unchanged builder
const guarded = sql`SELECT id FROM guarded_users`;
if (choose(guarded)) query(guarded); // opaque known-prefix
query(guarded); // opaque known-prefix
const imported = sql`SELECT id FROM imported_users`;
externalHelper(imported);
query(gate ? imported : sql`SELECT 3`); // opaque
const boxed = sql`SELECT id FROM boxed_users`;
unknownHelper({ statement: boxed, ...{ label: "box" } });
query(gate ? boxed : sql`SELECT 4`); // opaque
const array = sql`SELECT id FROM array_users`;
unknownHelper([null, , ...[array]]);
query(gate ? array : sql`SELECT 5`); // opaque
const spread = sql`SELECT id FROM spread_users`;
unknownHelper(...[spread]);
query(gate ? spread : sql`SELECT 6`); // opaque
const selected = sql`SELECT id FROM selected_users`;
unknownHelper(gate ? selected : null);
query(gate ? selected : sql`SELECT 7`); // opaque
const logical = sql`SELECT id FROM logical_users`;
unknownHelper(gate && logical);
query(gate ? logical : sql`SELECT 8`); // opaque
const sequence = sql`SELECT id FROM sequence_users`;
unknownHelper((gate, sequence));
query(gate ? sequence : sql`SELECT 9`); // opaque
let assigned = "initial";
unknownHelper(assigned = sql`SELECT id FROM assigned_users`);
query(gate ? assigned : sql`SELECT 10`); // opaque
const property = sql`SELECT id FROM property_users`;
const holder = {};
unknownHelper(holder.statement = property);
query(gate ? property : sql`SELECT 11`); // opaque
let replacement = sql`SELECT id FROM old_users`;
const previous = replacement;
unknownHelper(replacement, replacement = sql`SELECT id FROM new_users`);
query(previous); // opaque known-prefix
query(gate ? replacement : sql`SELECT 12`); // opaque
const scalar = "SELECT id FROM string_users";
let scalarCopy = scalar;
function chooseText(input: string): boolean {
  input += " OFFSET 11";
  return runtimeFlag;
}
query(chooseText(scalar) ? scalar : "SELECT 13"); // finite
query(scalar); // unchanged string
unknownHelper(scalarCopy += " LIMIT 1");
const pure = sql`SELECT id FROM pure_users`;
query(pure); // unchanged builder
query(gate ? pure : sql`SELECT 14`); // finite
function passthrough(input: SQLStatement): SQLStatement {
  return input;
}
query(gate ? passthrough(pure) : sql`SELECT 15`); // finite
const tail = sql` LIMIT 1`;
const appender = sql`SELECT id FROM append_users`;
appender.append(tail);
appender["append"](tail);
sql(tail);
sql.raw("SELECT 16");
sql.join([tail]);
query(gate ? tail : sql` LIMIT 2`); // finite
const outer = sql`SELECT id FROM outer_users`;
function independent(outer: SQLStatement): void {
  unknownHelper(outer);
}
query(gate ? outer : sql`SELECT 17`); // finite
const member = sql`SELECT id FROM member_users`;
const other = { append(input: SQLStatement): boolean {
  input.append(sql` OFFSET 12`);
  return runtimeFlag;
} };
query(other.append(member) ? member : sql`SELECT 18`); // opaque
query(member); // opaque
const interpolated = sql`SELECT id FROM interpolation_users`;
query(sql`SELECT ${externalHelper(interpolated)} FROM (${interpolated})`); // opaque template
const computedMember = sql`SELECT id FROM computed_users`;
other["append"](computedMember);
query(gate ? computedMember : sql`SELECT 19`); // opaque
const primitive = "SELECT id FROM primitive_users";
unknownHelper({ primitive });
query(gate ? primitive : "SELECT 20"); // finite
const ordinary = sql`SELECT id FROM ordinary_users`;
unknownHelper(ordinary);
query(ordinary); // opaque known-prefix
query(gate ? ordinary : sql`SELECT 21`); // opaque
const captured = sql`SELECT id FROM captured_users`;
const capturedAlias = captured;
function callback() {
  unknownHelper(captured);
  query(gate ? captured : sql`SELECT 22`); // opaque
  query(captured); // opaque known-prefix
  const localAlias = capturedAlias;
  query(choose(localAlias) ? localAlias : sql`SELECT 23`); // opaque
  query(capturedAlias); // opaque
}
query(gate ? captured : sql`SELECT 24`); // finite
query(gate ? capturedAlias : sql`SELECT 25`); // finite
const switchCaptured = sql`SELECT id FROM switch_users`;
const switchAlias = switchCaptured;
function unusedSwitchCallback() {
  switch (choose(switchCaptured)) {
    case true:
      query(switchCaptured); // opaque
      break;
  }
  query(switchAlias); // opaque
}
query(switchCaptured); // unchanged builder
query(switchAlias); // opaque legacy-alias
query(gate ? switchCaptured : sql`SELECT 26`); // finite
query(gate ? switchAlias : sql`SELECT 27`); // finite
const capturedChoice = gate ? sql`SELECT 28` : sql`SELECT 29`;
function unusedChoiceCallback() {
  unknownHelper(capturedChoice);
  query(gate ? capturedChoice : sql`SELECT 30`); // opaque
}
query(gate ? capturedChoice : sql`SELECT 31`); // finite
const captureSource = sql`SELECT id FROM capture_users`;
let uncertainAlias = captureSource;
if (gate) uncertainAlias = captureSource;
function unusedAliasCallback() {
  unknownHelper(captureSource);
  query(gate ? uncertainAlias : sql`SELECT 32`); // opaque
}
query(gate ? captureSource : sql`SELECT 33`); // finite
query(gate ? uncertainAlias : sql`SELECT 34`); // finite
function shadowedAliasCallback() {
  const uncertainAlias = sql`SELECT id FROM shadow_capture_users`;
  unknownHelper(captureSource);
  query(gate ? uncertainAlias : sql`SELECT 35`); // finite
}
query(gate ? uncertainAlias : sql`SELECT 36`); // finite
