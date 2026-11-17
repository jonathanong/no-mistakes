import { writePool, type QueryExecutor } from "@example/psql";
import sql from "sql-template-strings";

export function visibleDefaultTag(partition: string, query: QueryExecutor = writePool) {
  return query(sql`SELECT * FROM t WHERE partition = ${partition}`);
}

export function hiddenDefaultTag(where: unknown, query: QueryExecutor = writePool) {
  return query(sql`SELECT * FROM t WHERE ${where}`);
}

export function hiddenDestructuredParameter(
  { where }: { where: unknown },
  query: QueryExecutor = writePool,
) {
  return query(sql`SELECT * FROM t WHERE ${where}`);
}

export function hiddenDestructuredLocal(
  opts: { where: unknown },
  query: QueryExecutor = writePool,
) {
  const { where } = opts;
  return query(sql`SELECT * FROM t WHERE ${where}`);
}

export function hiddenNested(
  opts: { where: { sql: unknown }; sql: number },
  query: QueryExecutor = writePool,
) {
  const { where } = opts;
  return query(sql`SELECT * FROM t WHERE ${where.sql}`);
}

export function scalarMethod(partition: string, query: QueryExecutor = writePool) {
  return query(
    sql`SELECT * FROM t WHERE partition = ${partition.slice(0)} AND n = ${partition.length}`,
  );
}

export function fragmentMethod(where: string, query: QueryExecutor = writePool) {
  return query(sql`SELECT * FROM t WHERE ${where.toString()}`);
}

export function renamedField(
  { where: clause }: { where: unknown },
  query: QueryExecutor = writePool,
) {
  return query(sql`SELECT * FROM t WHERE ${clause}`);
}

export function multiField(
  { noise, where }: { noise: number; where: unknown },
  query: QueryExecutor = writePool,
) {
  return query(sql`SELECT * FROM t WHERE ${where}`);
}

export function defaultField(
  { where = sql`id = 1` }: { where?: unknown },
  query: QueryExecutor = writePool,
) {
  return query(sql`SELECT * FROM t WHERE ${where}`);
}

export function computedField(
  { ["where"]: where }: { where: unknown },
  query: QueryExecutor = writePool,
) {
  return query(sql`SELECT * FROM t WHERE ${where}`);
}

export function mutatedField(
  opts: { where: unknown },
  fragment: unknown,
  query: QueryExecutor = writePool,
) {
  opts.where = fragment;
  return query(sql`SELECT * FROM t WHERE ${opts.where}`);
}

export function mutatedAliasField(
  opts: { where: unknown },
  fragment: unknown,
  query: QueryExecutor = writePool,
) {
  const alias = opts;
  alias.where = fragment;
  return query(sql`SELECT * FROM t WHERE ${opts.where}`);
}

export function assignedField(
  opts: { where: unknown },
  fragment: unknown,
  query: QueryExecutor = writePool,
) {
  Object.assign(opts, { where: fragment });
  return query(sql`SELECT * FROM t WHERE ${opts.where}`);
}

export function castAssignedField(
  opts: { where: unknown },
  fragment: unknown,
  query: QueryExecutor = writePool,
) {
  Object.assign(opts as object, { where: fragment });
  return query(sql`SELECT * FROM t WHERE ${opts.where}`);
}

export function castAliasField(
  opts: { where: unknown },
  fragment: unknown,
  query: QueryExecutor = writePool,
) {
  const alias = opts as { where: unknown };
  alias.where = fragment;
  return query(sql`SELECT * FROM t WHERE ${opts.where}`);
}

export function boxedField(opts: { where: unknown }, fragment: unknown, query: QueryExecutor) {
  const box = { opts };
  box.opts.where = fragment;
  return query(sql`SELECT * FROM t WHERE ${opts.where}`);
}

export function methodField(opts: { where: unknown; mutate(): void }, query: QueryExecutor) {
  opts.mutate();
  return query(sql`SELECT * FROM t WHERE ${opts.where}`);
}

export function repeatedField(opts: { where: number }, query: QueryExecutor) {
  const { where: value } = opts;
  return query(sql`SELECT * FROM t WHERE id = ${value} AND other_id = ${value}`);
}

export function scalarFieldMethod(opts: { where: number }, query: QueryExecutor) {
  return query(sql`SELECT * FROM t WHERE id = ${opts.where.toString()}`);
}

export function optionBag(opts: { query: QueryExecutor; where: unknown }) {
  return opts.query(sql`SELECT * FROM t WHERE ${opts.where}`);
}

export function nestedMember(
  opts: { inner: { where: unknown }; where: number },
  query: QueryExecutor = writePool,
) {
  return query(sql`SELECT * FROM t WHERE ${opts.inner.where}`);
}

export function directNested(
  { where }: { where: { sql: unknown } },
  query: QueryExecutor = writePool,
) {
  return query(sql`SELECT * FROM t WHERE ${where.sql}`);
}

export function writtenDestructured(
  { where }: { where: unknown },
  fragment: unknown,
  query: QueryExecutor = writePool,
) {
  where = fragment;
  return query(sql`SELECT * FROM t WHERE ${where}`);
}

export function writtenLocal(
  opts: { where: unknown },
  fragment: unknown,
  query: QueryExecutor = writePool,
) {
  let { where } = opts;
  where = fragment;
  return query(sql`SELECT * FROM t WHERE ${where}`);
}

export function computedAlias(opts: { where: unknown }, query: QueryExecutor = writePool) {
  const { ["where"]: where } = opts;
  return query(sql`SELECT * FROM t WHERE ${where}`);
}

export function loopDestructured(
  fragments: Array<{ where: unknown }>,
  query: QueryExecutor = writePool,
) {
  for (const { where } of fragments) query(sql`SELECT * FROM t WHERE ${where}`);
}

export function arrayElision(values: [unknown, unknown], query: QueryExecutor = writePool) {
  const [, where] = values;
  return query(sql`SELECT * FROM t WHERE ${where}`);
}

export function repeatedInterpolation(id: number, query: QueryExecutor) {
  return query(sql`SELECT * FROM t WHERE id = ${id || id}`);
}

export function quotedField({ "where-key": value }: { "where-key": number }, query: QueryExecutor) {
  return query(sql`SELECT * FROM t WHERE id = ${value}`);
}

export function uninitializedScalar(query: QueryExecutor) {
  let value: number | undefined;
  return query(sql`SELECT * FROM t WHERE id = ${value}`);
}

export function nestedParameter(query: QueryExecutor) {
  const execute = (fragment: unknown) => query(sql`SELECT * FROM t WHERE ${fragment}`);
  return execute(sql`id = 1`);
}

export function twoQueries(opts: { query: QueryExecutor; where: unknown }) {
  opts.query(sql`SELECT * FROM t WHERE ${opts.where}`);
  return opts.query(sql`SELECT * FROM u WHERE ${opts.where}`);
}

export function differentReceiver(
  opts: { where: unknown; query(): void },
  db: { query: QueryExecutor },
) {
  opts.query();
  return db.query(sql`SELECT * FROM t WHERE ${opts.where}`);
}
