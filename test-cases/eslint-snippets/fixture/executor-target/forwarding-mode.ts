import { read, write, type QueryExecutor } from "@example/db";
import { sql, raw } from "@example/sql";

function visible(query: QueryExecutor) {
  return query("SELECT id FROM t");
}
function visibleUntyped(query) {
  return query("SELECT id FROM t");
}
function visibleBag({ query }) {
  return query("SELECT id FROM t");
}
function visibleNamedBag(options) {
  return options.query("SELECT id FROM t WHERE id = $1", [options.id]);
}
function escapedBag(options) {
  consume(options);
  return options.query("SELECT id FROM t");
}
function visibleTagged(query) {
  return query(sql`SELECT id FROM t WHERE id = ${id}`);
}
function hiddenRawTemplate(query) {
  return query(`SELECT ${fragment} FROM t`);
}
function hiddenUntrustedTag(query) {
  return query(raw`SELECT ${fragment} FROM t`);
}
function hidden(fragment, query) {
  return query(fragment);
}
function hiddenTyped(fragment: string, query: QueryExecutor) {
  return query(fragment);
}
function hiddenBag(fragment, { query }) {
  return query(fragment);
}
function opaque(query: QueryExecutor) {
  arguments[0](fragment);
  return query("SELECT id FROM t");
}

visible(read);
visibleUntyped(read);
visibleUntyped(flag ? read : write);
visibleBag({ query: read });
visibleBag({ query: flag ? read : write });
visibleNamedBag({ query: read, id });
escapedBag({ query: read });
visibleTagged(read);
hiddenRawTemplate(read);
hiddenUntrustedTag(read);
hidden(sql`SELECT ${fragment} FROM t`, read);
hidden(sql`SELECT ${fragment} FROM t`, flag ? read : write);
hiddenTyped(sql`SELECT ${fragment} FROM t`, read);
hiddenBag(sql`SELECT ${fragment} FROM t`, { query: read });
opaque(read);
const keys = new Map();
keys.set(read, true);
load((statement) => read(statement));
load((statement) => read(statement, extra));
