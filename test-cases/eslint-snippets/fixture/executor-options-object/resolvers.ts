import { type SessionExecutor } from "@example/db";
import sql from "@example/sql";

export type RejectOptions = { query: SessionExecutor };

export function rejectDeclared(id: string, options: RejectOptions) {
  return options.query(sql`SELECT ${id}`);
}

export function rejectOptionalParameter(id: string, options?: RejectOptions) {
  return options?.query(sql`SELECT ${id}`);
}

export function rejectUnioned(id: string, options: RejectOptions | undefined | null) {
  return options?.query(sql`SELECT ${id}`);
}

export function rejectDefaulted(id: string, options: RejectOptions = invalidDefault()) {
  return options.query(sql`SELECT ${id}`);
}

export function rejectDestructured(id: string, { query }: RejectOptions) {
  return query(sql`SELECT ${id}`);
}

declare function invalidDefault<T = never>(): T;

export function rejectUntyped(id: string, options) {
  return options.query(sql`SELECT ${id}`);
}

export function rejectAny(id: string, options: any) {
  return options.query(sql`SELECT ${id}`);
}

export function rejectUnknown(id: string, options: { query: unknown }) {
  return (options.query as SessionExecutor)(sql`SELECT ${id}`);
}

export function rejectFunction(id: string, options: { query: (statement: unknown) => unknown }) {
  return options.query(sql`SELECT ${id}`);
}

export function rejectOptionalProperty(id: string, options: { query?: SessionExecutor }) {
  return options.query?.(sql`SELECT ${id}`);
}

export function rejectInferred(id: string, options = { query: invalidDefault<SessionExecutor>() }) {
  return options.query(sql`SELECT ${id}`);
}

export function rejectUnion(
  id: string,
  options: RejectOptions | { query: SessionExecutor; extra: 1 },
) {
  return options.query(sql`SELECT ${id}`);
}

const stash: unknown[] = [];
export function rejectEscaping(id: string, options?: RejectOptions) {
  stash.push(options);
  return options?.query(sql`SELECT ${id}`);
}
