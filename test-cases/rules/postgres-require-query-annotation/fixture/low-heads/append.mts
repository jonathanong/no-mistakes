import { read } from "@bench/db";
import sql from "any-builder";
read(sql`/* inline */ SELECT 1`.append(sql` AND x = ${value}`).append(" suffix")); // low: true
read(sql`SELECT 1`.append(sql`/* later */ SELECT 2`)); // low: false
read((sql`/* wrapped */ SELECT 1` as Statement).append("suffix")); // low: true
read(sql`/* optional */ SELECT 1`.append?.("suffix")); // low: false
read(sql`/* computed */ SELECT 1`[method]("suffix")); // low: false
export function local() {
  const query = sql`/* local */ SELECT 1`;
  query.append(sql` AND x = ${value}`);
  read(query); // low: true
  read(query.append(" suffix").append(" more")); // low: true
  escape(query);
  read(query); // low: false
  const string = "/* string */ SELECT 1".append("");
  string.append("suffix");
  read(string); // low: true
  escape(string);
  read(string); // low: false
  let changed = sql`/* changed */ SELECT 1`;
  changed.append("suffix");
  changed = replacement;
  read(changed); // low: false
  const alias = sql`/* alias */ SELECT 1`;
  alias.append("suffix");
  const other = alias;
  read(alias); // low: false
  const nested = sql`/* nested */ SELECT 1`;
  nested.append(read(sql`SELECT unnamed`)); // low: false
  read(nested); // low: false
}
export function captured() {
  const query = sql`/* captured */ SELECT 1`;
  query.append("suffix");
  read(query); // low: false
  // A hoisted function can run before the earlier executor despite appearing later.
  function mutate() { query.prepend("SELECT unsafe"); }
}
export function nestedExecution() {
  const query = sql`/* outer */ SELECT 1`;
  query.append("suffix");
  function execute() { read(query); } // low: false
}
export function shadowed() {
  const query = sql`/* outer */ SELECT 1`;
  query.append("suffix");
  { const query = sql`SELECT unnamed`; read(query.append("suffix")); } // low: false
  read(query); // low: true
}
export function initializer() {
  const query = sql`/* initialized */ SELECT 1`.append("suffix");
  read(query); // low: true
  const unknown = sql`/* unknown */ SELECT 1`;
  unknown.append(mutator(unknown));
  read(unknown); // low: false
}
const exported = sql`/* exported */ SELECT 1`;
export { exported };
exported.append("suffix");
read(exported); // low: false

export function repeated() {
  const query = sql`/* repeated */ SELECT 1`;
  query.append("suffix");
  // The later escape precedes the next iteration's executor call.
  for (const item of items) { read(query); escape(query); } // low: false
}
