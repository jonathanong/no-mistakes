import { read } from "db";
import { ALLOWED_TABLES } from "./tables.ts";
type Config = { schema: (typeof ALLOWED_TABLES)[number]; toString(): string };

export function projectedString(config: Config) {
  if (!ALLOWED_TABLES.includes(config.schema)) throw new Error("Unknown schema");
  return read(`SELECT * FROM ${(`${config}` as unknown as Config).schema}`);
}

export function projectedObject(config: Config) {
  if (!ALLOWED_TABLES.includes(config.schema)) throw new Error("Unknown schema");
  return read(`SELECT * FROM ${{ schema: config }.schema}`);
}

export function projectedArray(config: Config) {
  if (!ALLOWED_TABLES.includes(config.schema)) throw new Error("Unknown schema");
  return read(`SELECT * FROM ${[config][0]}`);
}
