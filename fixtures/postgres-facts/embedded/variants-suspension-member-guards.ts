import { query } from "@example/db";
import sql from "sql-template-strings";

type Mode = "all" | "mine" | "team";
declare function pause(): Promise<void>;

export async function memberAwait(options: { mode: Mode }) {
  const statement = sql`SELECT 1`;
  if (options.mode === "mine") statement.append(sql` /* mine */`);
  await pause();
  if (options.mode === "team") statement.append(sql` /* team */`);
  query(statement);
}

export function* memberYield(options: { mode: Mode }) {
  const statement = sql`SELECT 1`;
  if (options.mode === "mine") statement.append(sql` /* mine */`);
  yield;
  if (options.mode === "team") statement.append(sql` /* team */`);
  query(statement);
}

export async function aliasAwait(options: { mode: Mode }) {
  const mode = options.mode;
  const statement = sql`SELECT 1`;
  await pause();
  if (mode === "mine" || mode === "all") statement.append(sql` /* mine */`);
  if (mode === "team" || mode === "all") statement.append(sql` /* team */`);
  query(statement);
}

export function* aliasYield(options: { mode: Mode }) {
  const mode = options.mode;
  const statement = sql`SELECT 1`;
  yield;
  if (mode === "mine" || mode === "all") statement.append(sql` /* mine */`);
  if (mode === "team" || mode === "all") statement.append(sql` /* team */`);
  query(statement);
}

export async function aliasAfterAwait(options: { mode: Mode }) {
  await pause();
  const mode = options.mode;
  const statement = sql`SELECT 1`;
  if (mode === "mine") statement.append(sql` /* mine */`);
  if (mode === "team") statement.append(sql` /* team */`);
  query(statement);
}

export async function guardsBeforeAwait(options: { mode: Mode }) {
  const statement = sql`SELECT 1`;
  if (options.mode === "mine" || options.mode === "all") statement.append(sql` /* mine */`);
  if (options.mode === "team" || options.mode === "all") statement.append(sql` /* team */`);
  await query(statement);
}

export function nonNullReceiver(options: { mode: Mode | "all"; 1: Mode }) {
  const statement = sql`SELECT 1`;
  if (options!.mode === "mine" || options!.mode === "all") statement.append(sql` /* mine */`);
  if (options!.mode === "team" || options!.mode === "all") statement.append(sql` /* team */`);
  query(statement);
}
