import sql from "@example/sql";
import { read, type SessionExecutor } from "@example/db";
import type { Input } from "./local";

function byId(query: SessionExecutor, id: string) {
  return query(sql`SELECT id FROM sample WHERE id = ${id}`);
}

function matchesKey(input: Readonly<Input>, expected: string) {
  const { id: key, "other": region } = input;
  const prefix = region === "" ? `default` : region;
  return `${prefix}:${key}` === expected;
}

function matchesThreshold(input: Readonly<Input>, score: number, limit: number | null) {
  const adjusted = (score * 2 - 1);
  const enabled = true;
  const disabled = false;
  if (limit !== null) {
    return adjusted >= limit && input.id !== "";
  } else {
    return enabled && !disabled && input.other !== "";
  }
}

export function projectionAfterDestructuredReader(input: Input) {
  matchesKey(input, "default:sample");
  return byId(read, input.id);
}

export function projectionAfterScalarReader(input: Input) {
  matchesThreshold(input, 1, null);
  return byId(read, input.id);
}

function typedReader(input: Input) {
  const absent = void input.other;
  return typeof input.id === "string" && typeof absent === "undefined";
}

export function acceptsTypeCheck(input: Input) {
  typedReader(input);
  return byId(read, input.id);
}
