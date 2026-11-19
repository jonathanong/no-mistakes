import { read, write, type SessionExecutor } from "@example/db";
import sql, { type SqlStatement } from "@example/sql";
import { lookup } from "./other";

interface QueryOptions {
  label?: string;
  query?: SessionExecutor;
  readOnly?: boolean;
}
const stash: QueryOptions[] = [];
const shared: QueryOptions = {};

async function notConfigured(id: string, options: QueryOptions = {}) {
  await lookup(sql`SELECT id FROM widgets WHERE id = ${id}`, options);
}

async function mutatedFirst(id: string, options: QueryOptions = {}) {
  options.readOnly = true;
  await write(sql`SELECT id FROM widgets WHERE id = ${id}`, options);
}

async function storedFirst(id: string, options: QueryOptions = {}) {
  stash.push(options);
  await write(sql`SELECT id FROM widgets WHERE id = ${id}`, options);
}

async function overrideLiteralExecutor(id: string, options: QueryOptions = {}) {
  await write(sql`SELECT id FROM widgets WHERE id = ${id}`, {
    ...options,
    query: (statement) => read(statement),
  });
}

async function overrideUnproven(id: string, other: SessionExecutor, options: QueryOptions = {}) {
  await write(sql`SELECT id FROM widgets WHERE id = ${id}`, { ...options, query: other });
}

async function spreadNotFirst(id: string, options: QueryOptions = {}) {
  await write(sql`SELECT id FROM widgets WHERE id = ${id}`, { readOnly: false, ...options });
}

async function dynamicStatement(statement: string, options: QueryOptions = {}) {
  await write(statement, options);
}

async function dynamicHole(filter: SqlStatement, options: QueryOptions = {}) {
  await write(sql`SELECT id FROM widgets WHERE ${filter}`, options);
}

async function selectWidget(id: string, execute: SessionExecutor) {
  return execute(sql`SELECT id FROM widgets WHERE id = ${id}`);
}

async function selectDynamic(column: string, execute: SessionExecutor) {
  return execute(`SELECT ${column} FROM widgets`);
}

async function manyStatements(id: string, options: QueryOptions = {}) {
  return selectWidget(id, (statement) => {
    stash.push(options);
    return write(statement, options);
  });
}

async function transformsStatement(id: string, options: QueryOptions = {}) {
  return selectWidget(id, (statement) => write(`${statement} FOR UPDATE`, options));
}

async function closureDynamicSql(column: string, options: QueryOptions = {}) {
  return selectDynamic(column, (statement) => write(statement, options));
}

export async function closureLocalOptions(id: string) {
  return selectWidget(id, (statement) => write(statement, shared));
}

async function closureOverride(id: string, other: SessionExecutor, options: QueryOptions = {}) {
  return selectWidget(id, (statement) => write(statement, { ...options, query: other }));
}

async function closureNotConfigured(id: string, options: QueryOptions = {}) {
  return selectWidget(id, (statement) => lookup(statement, options));
}

export async function reports(query: SessionExecutor, id: string) {
  await notConfigured(id, { query });
  await mutatedFirst(id, { query });
  await storedFirst(id, { query });
  await overrideLiteralExecutor(id, { query });
  await overrideUnproven(id, query, { query });
  await spreadNotFirst(id, { query });
  await dynamicStatement(id, { query });
  await dynamicHole(sql`state = 'sent'`, { query });
  await manyStatements(id, { query });
  await transformsStatement(id, { query });
  await closureDynamicSql(id, { query });
  await closureOverride(id, query, { query });
  await closureNotConfigured(id, { query });
}
