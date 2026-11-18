import check, { unchecked, unknownCheck } from "@example/assertions";
import { read } from "@example/db";
import { namedCheck as verify } from "./configured-helpers";
import { byId, type Input } from "./query";

export function afterShadowedImport(input: Input, check: (value: unknown) => void) {
  check(input);
  return byId(read, input.id);
}

export function afterDifferentModule(input: Input) {
  verify(input);
  return byId(read, input.id);
}

export function afterCapturedImport(input: Input) {
  const captured: typeof check = check;
  captured(input);
  return byId(read, input.id);
}

export function afterAnyImport(input: Input) {
  unchecked(input);
  return byId(read, input.id);
}

export function afterUnknownImport(input: Input) {
  unknownCheck(input);
  return byId(read, input.id);
}

export function withRepeatedArgument(input: Input) {
  check(input, input);
  return byId(read, input.id);
}

export function withSpread(input: Input) {
  check(...([input] as const));
  return byId(read, input.id);
}

export function afterConfiguredFieldWrite(input: Input) {
  check(input);
  input.id = "changed";
  return byId(read, input.id);
}
