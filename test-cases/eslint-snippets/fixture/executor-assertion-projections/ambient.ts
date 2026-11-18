import { read } from "@example/db";
import { byId, type Input } from "./query";

declare function ensure(value: unknown, status?: number, message?: string): asserts value;

export function beforeProjection(input: Input | undefined) {
  ensure(input, 400, "Input required");
  return byId(read, input.id);
}

declare function ensureInput(value: unknown): asserts value is Input;

export function beforeTypeNarrowedProjection(input: unknown) {
  ensureInput(input);
  return byId(read, input.id);
}
