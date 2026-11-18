import { query } from "@example/db";
import sql from "sql-template-strings";

type Mode = "all" | "mine" | "team";
type OpenMode = string;
type GenericMode<T = string> = T;
type WideMode =
  | "all" | "mine" | "team" | "a" | "b" | "c" | "d" | "e" | "f"
  | "g" | "h" | "i" | "j" | "k" | "l" | "m" | "n";

declare namespace Modes {
  export type Mode = "all" | "mine" | "team";
}

declare function markParameter(
  target: object,
  key: string | symbol | undefined,
  index: number,
): void;

export function optionalParameter(options?: { mode: Mode }) {
  const statement = sql`SELECT 1`;
  if (options!.mode === "mine") statement.append(sql` /* mine */`);
  if (options!.mode === "team") statement.append(sql` /* team */`);
  query(statement);
}

export function defaultParameter(options: { mode: Mode } = { mode: "all" }) {
  const statement = sql`SELECT 1`;
  if (options.mode === "mine") statement.append(sql` /* mine */`);
  if (options.mode === "team") statement.append(sql` /* team */`);
  query(statement);
}

export function destructuredParameter({ options }: { options: { mode: Mode } }) {
  const statement = sql`SELECT 1`;
  if (options.mode === "mine") statement.append(sql` /* mine */`);
  if (options.mode === "team") statement.append(sql` /* team */`);
  query(statement);
}

export function optionalProperty(options: { mode?: Mode }) {
  const statement = sql`SELECT 1`;
  if (options.mode === "mine") statement.append(sql` /* mine */`);
  if (options.mode === "team") statement.append(sql` /* team */`);
  query(statement);
}

export function computedProperty(options: { ["mode"]: Mode }) {
  const statement = sql`SELECT 1`;
  if (options.mode === "mine") statement.append(sql` /* mine */`);
  if (options.mode === "team") statement.append(sql` /* team */`);
  query(statement);
}

export function nonStringUnion(options: { mode: Mode | number }) {
  const statement = sql`SELECT 1`;
  if (options.mode === "mine") statement.append(sql` /* mine */`);
  if (options.mode === "team") statement.append(sql` /* team */`);
  query(statement);
}

export function openAlias(options: { mode: OpenMode }) {
  const statement = sql`SELECT 1`;
  if (options.mode === "mine") statement.append(sql` /* mine */`);
  if (options.mode === "team") statement.append(sql` /* team */`);
  query(statement);
}

export function genericAlias(options: { mode: GenericMode }) {
  const statement = sql`SELECT 1`;
  if (options.mode === "mine") statement.append(sql` /* mine */`);
  if (options.mode === "team") statement.append(sql` /* team */`);
  query(statement);
}

export function namespaceAlias(options: { mode: Modes.Mode }) {
  const statement = sql`SELECT 1`;
  if (options.mode === "mine") statement.append(sql` /* mine */`);
  if (options.mode === "team") statement.append(sql` /* team */`);
  query(statement);
}

export function wideAlias(options: { mode: WideMode }) {
  const statement = sql`SELECT 1`;
  if (options.mode === "mine") statement.append(sql` /* mine */`);
  if (options.mode === "team") statement.append(sql` /* team */`);
  query(statement);
}

export function genericShadow<Mode extends string>(options: { mode: Mode }) {
  const statement = sql`SELECT 1`;
  if (options.mode === "mine") statement.append(sql` /* mine */`);
  if (options.mode === "team") statement.append(sql` /* team */`);
  query(statement);
}

export class QueryRunner {
  decoratedParameter(@markParameter options: { mode: Mode }) {
    const statement = sql`SELECT 1`;
    if (options.mode === "mine") statement.append(sql` /* mine */`);
    if (options.mode === "team") statement.append(sql` /* team */`);
    query(statement);
  }
}

export function laterAliasShadow() {
  function inner(options: { mode: Mode }) {
    const statement = sql`SELECT 1`;
    if (options.mode === "mine") statement.append(sql` /* mine */`);
    if (options.mode === "team") statement.append(sql` /* team */`);
    query(statement);
  }

  type Mode = string;
  return inner;
}

export function receiverMethod(options: { mode: Mode; flip(): void }) {
  const statement = sql`SELECT 1`;
  if (options.mode === "mine") statement.append(sql` /* mine */`);
  options.flip();
  if (options.mode === "team") statement.append(sql` /* team */`);
  query(statement);
}
