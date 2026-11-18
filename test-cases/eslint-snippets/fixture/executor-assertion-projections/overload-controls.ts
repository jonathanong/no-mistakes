import { read } from "@example/db";
import ensure from "@example/guard-overloads";
import { writing, mixed, differentIndex, runtimeMerged } from "./overload-helpers";
import { byId, type Input } from "./query";

export function writingOverload(input: Input) {
  writing(input, 404);
  return byId(read, input.id);
}

export function mixedOverload(input: Input) {
  mixed(input);
  return byId(read, input.id);
}

export function wrongIndexOverload(input: Input) {
  differentIndex(input, true);
  return byId(read, input.id);
}

export function withPropertiesRecord(input: Input) {
  ensure(input, 404, { detail: "Missing input" });
  return byId(read, input.id);
}

export function withRuntimeNamespace(input: Input) {
  runtimeMerged(input);
  return byId(read, input.id);
}
