import { read } from "@example/db";
import { requireValue } from "./helpers";
import type { requireValue as typeOnly } from "./helpers";
import { byId, type Input } from "./query";

const captured: typeof requireValue = requireValue;

export function afterCapturedBinding(input: Input) {
  captured(input);
  return byId(read, input.id);
}

export function afterTypeOnlyImport(input: Input) {
  typeOnly(input);
  return byId(read, input.id);
}

export function afterOptionalCall(input: Input) {
  requireValue?.(input);
  return byId(read, input.id);
}

export function withWrappedArgument(input: Input) {
  requireValue(input as Input);
  return byId(read, input.id);
}
