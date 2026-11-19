import { read, write, type SessionExecutor } from "@example/db";
import sql from "@example/sql";

interface QueryOptions {
  label?: string;
  query?: SessionExecutor;
  readOnly?: boolean;
}

async function passOptions(id: string, options: QueryOptions = {}) {
  await write(sql`UPDATE widgets SET state = 'sent' WHERE id = ${id}`, options);
}

async function spreadOverride(id: string, options: QueryOptions = {}) {
  await write(sql`UPDATE widgets SET state = 'sent' WHERE id = ${id}`, {
    ...options,
    readOnly: false,
    label: `sent`,
  });
}

async function optionalNoDefault(id: string, options?: QueryOptions) {
  await read(sql`SELECT id FROM widgets WHERE id = ${id}`, options);
}

async function selectWidget(id: string, execute: SessionExecutor, now = new Date()) {
  return execute(sql`SELECT id FROM widgets WHERE id = ${id}`, { readOnly: now > new Date(0) });
}

async function fromPrimary(id: string, options?: QueryOptions, now = new Date()) {
  return selectWidget(id, (statement) => write(statement, { ...options, readOnly: false }), now);
}

async function fromBlock(id: string, options: QueryOptions = {}) {
  return selectWidget(id, (statement) => {
    return write(statement, options);
  });
}

async function assertDelivered(id: string, options: QueryOptions = {}) {
  return fromPrimary(id, options);
}

export async function directTerminals(query: SessionExecutor, id: string) {
  await passOptions(id, { query });
  await spreadOverride(id, { query });
  await optionalNoDefault(id, { query });
}

export async function closureChain(query: SessionExecutor, id: string) {
  await fromPrimary(id, { query });
  await fromBlock(id, { query });
  await assertDelivered(id, { query });
}
