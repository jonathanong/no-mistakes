import { read, type QueryExecutor } from "@example/db";
import { read as subRead } from "@example/db/sub";
import { read as otherRead } from "@example/other";
import { sql, raw } from "@example/sql";

export function unionExecutor(query: QueryExecutor | typeof read) {
  query("SELECT 1");
}

export function typeofExecutor(query: typeof read) {
  query("SELECT 1");
}

export function mixedExecutor(query: QueryExecutor | ((statement: unknown) => unknown)) {
  query("SELECT 1");
}

export function visibleImported(query: QueryExecutor) {
  query("SELECT id FROM t");
}

export function hiddenImported(fragment: string, query: QueryExecutor) {
  query(fragment);
}

export function visibleBagImported({ query }: { query: QueryExecutor }) {
  query("SELECT id FROM t");
}

export function hiddenBagImported(fragment: string, { query }: { query: QueryExecutor }) {
  query(fragment);
}

export function visibleTaggedImported(query: QueryExecutor) {
  query(sql`SELECT id FROM t WHERE id = ${id}`);
}

export function hiddenTagImported(query: QueryExecutor) {
  query(raw`SELECT ${fragment} FROM t`);
}

export function visibleNamedBagImported(options: { query: QueryExecutor; id: number }) {
  options.query("SELECT id FROM t WHERE id = $1", [options.id]);
}

export function hiddenCallImported(fragment: string, { query }: { query: QueryExecutor }) {
  query.call(null, fragment);
}

export function hiddenApplyImported(fragment: string, { query }: { query: QueryExecutor }) {
  query.apply(null, [fragment]);
}

export function exactSource(query: typeof read) { query("SELECT 1"); }
export function subSource(query: typeof subRead) { query("SELECT 1"); }
export function otherSource(query: typeof otherRead) { query("SELECT 1"); }
