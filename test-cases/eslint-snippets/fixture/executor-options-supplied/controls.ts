import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

interface QueryOptions {
  label?: string;
  query?: SessionExecutor;
  readOnly?: boolean;
}
interface OtherOptions {
  label?: string;
  query?: SessionExecutor;
}

declare const external: any;
declare function declared(id: string, options: QueryOptions): Promise<void>;
const stash: unknown[] = [];

async function assertDelivered(id: string, options: QueryOptions = {}) {
  await options.query?.(sql`SELECT ${id}`);
}

async function assertOther(id: string, options: OtherOptions = {}) {
  await options.query?.(sql`SELECT ${id}`);
}

async function assertDestructured(id: string, { query }: QueryOptions = {}) {
  await query?.(sql`SELECT ${id}`);
}

async function assertUntyped(id: string, options) {
  await options.query?.(sql`SELECT ${id}`);
}

async function assertRest(...args: [string, QueryOptions?]) {
  await args[1]?.query?.(sql`SELECT ${args[0]}`);
}

async function assertFewer(id: string) {
  await sql`SELECT ${id}`;
}

async function assertOverloaded(id: string, options?: QueryOptions): Promise<void>;
async function assertOverloaded(id: string, options: QueryOptions = {}) {
  await options.query?.(sql`SELECT ${id}`);
}

const assertArrow = async (id: string, options: QueryOptions = {}) => {
  await options.query?.(sql`SELECT ${id}`);
};

class Holder {
  constructor(id: string, options: QueryOptions = {}) {
    void options.query?.(sql`SELECT ${id}`);
  }
}

export async function absent(query: SessionExecutor, id: string) {
  async function reject(id: string, options: QueryOptions = {}) {
    await assertDelivered(id, options);
  }
  return reject(id, { label: query });
}

export async function nonExecutorValue(query: SessionExecutor, id: string) {
  async function reject(id: string, options: QueryOptions = {}) {
    await assertDelivered(id, options);
  }
  return reject(id, { query: (statement) => query(statement) });
}

async function rejectMutated(id: string, options: QueryOptions = {}) {
  options.readOnly = true;
  await assertDelivered(id, options);
}

async function rejectSpread(id: string, options: QueryOptions = {}) {
  await assertDelivered(id, { ...options });
}

async function rejectStored(id: string, options: QueryOptions = {}) {
  stash.push(options);
  await assertDelivered(id, options);
}

async function rejectDifferent(id: string, options: QueryOptions = {}) {
  await assertOther(id, options);
}

async function pingPong(id: string, options: QueryOptions = {}): Promise<void> {
  await pongPing(id, options);
}

async function pongPing(id: string, options: QueryOptions = {}): Promise<void> {
  await pingPong(id, options);
}

async function levelOne(id: string, options: QueryOptions = {}) {
  await levelTwo(id, options);
}
async function levelTwo(id: string, options: QueryOptions = {}) {
  await levelThree(id, options);
}
async function levelThree(id: string, options: QueryOptions = {}) {
  await levelFour(id, options);
}
async function levelFour(id: string, options: QueryOptions = {}) {
  await levelFive(id, options);
}
async function levelFive(id: string, options: QueryOptions = {}) {
  await assertDelivered(id, options);
}

async function rejectDestructured(id: string, options: QueryOptions = {}) {
  await assertDestructured(id, options);
}
async function rejectUntyped(id: string, options: QueryOptions = {}) {
  await assertUntyped(id, options);
}
async function rejectRest(id: string, options: QueryOptions = {}) {
  await assertRest(id, options);
}
async function rejectFewer(id: string, options: QueryOptions = {}) {
  await assertFewer(id, options);
}
async function rejectOverloaded(id: string, options: QueryOptions = {}) {
  await assertOverloaded(id, options);
}
async function rejectArrow(id: string, options: QueryOptions = {}) {
  await assertArrow(id, options);
}
async function rejectHolder(id: string, options: QueryOptions = {}) {
  new Holder(id, options);
}
async function rejectSpreadArgs(ids: [string], options: QueryOptions = {}) {
  await assertDelivered(...ids, options);
}
async function rejectDeclared(id: string, options: QueryOptions = {}) {
  await declared(id, options);
}
async function rejectExternal(id: string, options: QueryOptions = {}) {
  await external(id, options);
}

export async function forwarding(query: SessionExecutor, id: string) {
  await rejectMutated(id, { query });
  await rejectSpread(id, { query });
  await rejectStored(id, { query });
  await rejectDifferent(id, { query });
  await pingPong(id, { query });
  await levelOne(id, { query });
  await rejectDestructured(id, { query });
  await rejectUntyped(id, { query });
  await rejectRest(id, { query });
  await rejectFewer(id, { query });
  await rejectOverloaded(id, { query });
  await rejectArrow(id, { query });
  await rejectHolder(id, { query });
  await rejectSpreadArgs([id], { query });
  await rejectDeclared(id, { query });
  await rejectExternal(id, { query });
}
