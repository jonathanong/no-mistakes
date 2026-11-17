import { readPool, writePool, type QueryExecutor } from "@example/psql";
import sql from "sql-template-strings";

function visibleDefaultTag(partition: string, query: QueryExecutor = writePool) {
  return query(sql`SELECT * FROM t WHERE partition = ${partition}`);
}

function hiddenDefaultTag(where: unknown, query: QueryExecutor = writePool) {
  return query(sql`SELECT * FROM t WHERE ${where}`);
}

function hiddenDestructuredParameter(
  { where }: { where: unknown },
  query: QueryExecutor = writePool,
) {
  return query(sql`SELECT * FROM t WHERE ${where}`);
}

function hiddenDestructuredLocal(opts: { where: unknown }, query: QueryExecutor = writePool) {
  const { where } = opts;
  return query(sql`SELECT * FROM t WHERE ${where}`);
}

function hiddenNested(
  opts: { where: { sql: unknown }; sql: number },
  query: QueryExecutor = writePool,
) {
  const { where } = opts;
  return query(sql`SELECT * FROM t WHERE ${where.sql}`);
}

function scalarMethod(partition: string, query: QueryExecutor = writePool) {
  return query(
    sql`SELECT * FROM t WHERE partition = ${partition.slice(0)} AND n = ${partition.length}`,
  );
}

function fragmentMethod(where: string, query: QueryExecutor = writePool) {
  return query(sql`SELECT * FROM t WHERE ${where.toString()}`);
}

visibleDefaultTag("west", readPool);
hiddenDefaultTag(sql`partition = 'west'`, readPool);
hiddenDestructuredParameter({ where: sql`partition = 'west'` }, readPool);
hiddenDestructuredLocal({ where: sql`partition = 'west'` }, readPool);
hiddenDestructuredParameter({ where: 1 }, readPool);
hiddenDestructuredLocal({ where: 1 }, readPool);
hiddenNested({ where: { sql: sql`id = 1` }, sql: 1 }, readPool);
scalarMethod("west", readPool);
fragmentMethod(sql`id = 1`, readPool);
const scalarOptions = { where: 1 };
const scalarOptionsBase = { where: 1 };
const scalarOptionsAlias = scalarOptionsBase;
hiddenDestructuredParameter(scalarOptionsAlias, readPool);
hiddenDestructuredLocal(scalarOptions, readPool);
const spreadOptions = { where: 1 };
hiddenDestructuredParameter({ ...spreadOptions }, readPool);
hiddenDestructuredParameter({ ["where"]: 1 }, readPool);
hiddenDestructuredParameter({ other: 1 }, readPool);
hiddenDestructuredParameter(opaqueOptions, readPool);

function renamedField({ where: clause }: { where: unknown }, query: QueryExecutor = writePool) {
  return query(sql`SELECT * FROM t WHERE ${clause}`);
}
function multiField(
  { noise, where }: { noise: number; where: unknown },
  query: QueryExecutor = writePool,
) {
  return query(sql`SELECT * FROM t WHERE ${where}`);
}
function defaultField(
  { where = sql`id = 1` }: { where?: unknown },
  query: QueryExecutor = writePool,
) {
  return query(sql`SELECT * FROM t WHERE ${where}`);
}
function computedField({ ["where"]: where }: { where: unknown }, query: QueryExecutor = writePool) {
  return query(sql`SELECT * FROM t WHERE ${where}`);
}
renamedField({ where: sql`id = 1` }, readPool);
renamedField({ where: 1 }, readPool);
multiField({ noise: 1, where: sql`id = 1` }, readPool);
multiField({ noise: 1, where: 1 }, readPool);
defaultField({}, readPool);
computedField({ where: sql`id = 1` }, readPool);
const mutatedOptions = { where: 1 };
mutatedOptions.where = sql`id = 1`;
hiddenDestructuredParameter(mutatedOptions, readPool);
function mutatedField(
  opts: { where: unknown },
  fragment: unknown,
  query: QueryExecutor = writePool,
) {
  opts.where = fragment;
  return query(sql`SELECT * FROM t WHERE ${opts.where}`);
}
mutatedField({ where: 1 }, sql`id = 1`, readPool);
function mutatedAliasField(
  opts: { where: unknown },
  fragment: unknown,
  query: QueryExecutor = writePool,
) {
  const alias = opts;
  alias.where = fragment;
  return query(sql`SELECT * FROM t WHERE ${opts.where}`);
}
mutatedAliasField({ where: 1 }, sql`id = 1`, readPool);
function assignedField(
  opts: { where: unknown },
  fragment: unknown,
  query: QueryExecutor = writePool,
) {
  Object.assign(opts, { where: fragment });
  return query(sql`SELECT * FROM t WHERE ${opts.where}`);
}
assignedField({ where: 1 }, sql`id = 1`, readPool);
function castAssignedField(opts: { where: unknown }, fragment: unknown, query: QueryExecutor) {
  Object.assign(opts as object, { where: fragment });
  return query(sql`SELECT * FROM t WHERE ${opts.where}`);
}
function castAliasField(opts: { where: unknown }, fragment: unknown, query: QueryExecutor) {
  const alias = opts as { where: unknown };
  alias.where = fragment;
  return query(sql`SELECT * FROM t WHERE ${opts.where}`);
}
castAssignedField({ where: 1 }, sql`id = 1`, readPool);
castAliasField({ where: 1 }, sql`id = 1`, readPool);
function boxedField(opts: { where: unknown }, fragment: unknown, query: QueryExecutor) {
  const box = { opts };
  box.opts.where = fragment;
  return query(sql`SELECT * FROM t WHERE ${opts.where}`);
}
function methodField(opts: { where: unknown; mutate(): void }, query: QueryExecutor) {
  opts.mutate();
  return query(sql`SELECT * FROM t WHERE ${opts.where}`);
}
boxedField({ where: 1 }, sql`id = 1`, readPool);
methodField(
  {
    where: 1,
    mutate() {
      this.where = sql`id = 1`;
    },
  },
  readPool,
);
function repeatedField(opts: { where: number }, query: QueryExecutor) {
  const { where: value } = opts;
  return query(sql`SELECT * FROM t WHERE id = ${value} AND other_id = ${value}`);
}
function scalarFieldMethod(opts: { where: number }, query: QueryExecutor) {
  return query(sql`SELECT * FROM t WHERE id = ${opts.where.toString()}`);
}
repeatedField({ where: 1 }, readPool);
scalarFieldMethod({ where: 1 }, readPool);
function optionBag(opts: { query: QueryExecutor; where: unknown }) {
  return opts.query(sql`SELECT * FROM t WHERE ${opts.where}`);
}
optionBag({ query: readPool, where: 1 });
optionBag({ query: readPool, where: sql`id = 1` });
const reflectedOptions = { where: 1 };
Object.assign(reflectedOptions, { where: sql`id = 1` });
hiddenDestructuredParameter(reflectedOptions, readPool);
const aliasedOptions = { where: 1 };
const mutableAlias = aliasedOptions;
mutableAlias.where = sql`id = 1`;
hiddenDestructuredParameter(aliasedOptions, readPool);
function nestedMember(opts: { inner: { where: unknown }; where: number }, query: QueryExecutor) {
  return query(sql`SELECT * FROM t WHERE ${opts.inner.where}`);
}
function directNested({ where }: { where: { sql: unknown } }, query: QueryExecutor) {
  return query(sql`SELECT * FROM t WHERE ${where.sql}`);
}
function writtenDestructured(
  { where }: { where: unknown },
  fragment: unknown,
  query: QueryExecutor,
) {
  where = fragment;
  return query(sql`SELECT * FROM t WHERE ${where}`);
}
function writtenLocal(opts: { where: unknown }, fragment: unknown, query: QueryExecutor) {
  let { where } = opts;
  where = fragment;
  return query(sql`SELECT * FROM t WHERE ${where}`);
}
function computedAlias(opts: { where: unknown }, query: QueryExecutor) {
  const { ["where"]: where } = opts;
  return query(sql`SELECT * FROM t WHERE ${where}`);
}
nestedMember({ inner: { where: sql`id = 1` }, where: 1 }, readPool);
directNested({ where: { sql: sql`id = 1` } }, readPool);
writtenDestructured({ where: 1 }, sql`id = 1`, readPool);
writtenLocal({ where: 1 }, sql`id = 1`, readPool);
computedAlias({ where: sql`id = 1` }, readPool);
hiddenDestructuredParameter({ where: 1 }, readPool);
hiddenDestructuredParameter(makeOptions(), readPool);
hiddenDestructuredParameter({ where: 1, where: sql`id = 1` }, readPool);
function loopDestructured(fragments: Array<{ where: unknown }>, query: QueryExecutor) {
  for (const { where } of fragments) query(sql`SELECT * FROM t WHERE ${where}`);
}
function arrayElision(values: [unknown, unknown], query: QueryExecutor) {
  const [, where] = values;
  return query(sql`SELECT * FROM t WHERE ${where}`);
}
loopDestructured([{ where: sql`id = 1` }], readPool);
arrayElision([1, sql`id = 1`], readPool);
function repeatedInterpolation(id: number, query: QueryExecutor) {
  return query(sql`SELECT * FROM t WHERE id = ${id || id}`);
}
function quotedField({ "where-key": value }: { "where-key": number }, query: QueryExecutor) {
  return query(sql`SELECT * FROM t WHERE id = ${value}`);
}
function uninitializedScalar(query: QueryExecutor) {
  let value: number | undefined;
  return query(sql`SELECT * FROM t WHERE id = ${value}`);
}
function nestedParameter(query: QueryExecutor) {
  const execute = (fragment: unknown) => query(sql`SELECT * FROM t WHERE ${fragment}`);
  return execute(sql`id = 1`);
}
repeatedInterpolation(1, readPool);
quotedField({ "where-key": 1 }, readPool);
uninitializedScalar(readPool);
nestedParameter(readPool);
function twoQueries(opts: { query: QueryExecutor; where: unknown }) {
  opts.query(sql`SELECT * FROM t WHERE ${opts.where}`);
  return opts.query(sql`SELECT * FROM u WHERE ${opts.where}`);
}
twoQueries({ query: readPool, where: 1 });
twoQueries({ query: readPool, where: sql`id = 1` });
function differentReceiver(opts: { where: unknown; query(): void }, db: { query: QueryExecutor }) {
  opts.query();
  return db.query(sql`SELECT * FROM t WHERE ${opts.where}`);
}
differentReceiver(
  {
    where: 1,
    query() {
      this.where = sql`id = 1`;
    },
  },
  { query: readPool },
);
