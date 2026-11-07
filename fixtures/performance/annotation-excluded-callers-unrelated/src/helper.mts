import { query } from "@bench/db";

export async function read(statement: string): Promise<string> {
  return query(statement);
}

export async function run(statement: string, mode: number): Promise<string> {
  const result = await read(statement);
  return mode > 0 ? result : result.trim();
}
