import { read } from "@example/db";
import { byId, type Input } from "./query";

function literalGuard(value: unknown): asserts value {
  if (false) throw "unreachable";
}

function wrappedGuard(value: unknown): asserts value {
  if (!value) throw "Missing input";
  if (typeof value === "undefined") throw null;
}

function binaryGuard(value: unknown): asserts value {
  if (value === null || (typeof value !== "object" && value !== true)) throw 404;
}

function branches(value: unknown): asserts value {
  if (value) {
    return;
  } else {
    throw "Missing input";
  }
}

function emptyBranch(value: unknown): asserts value {
  if (value);
  else throw "Missing input";
}

export function afterEmptyBranch(input: Input) {
  emptyBranch(input);
  return byId(read, input.id);
}

export function afterLiteral(input: Input) {
  literalGuard(input);
  return byId(read, input.id);
}

export function afterWrapped(input: Input) {
  wrappedGuard(input);
  return byId(read, input.id);
}

export function afterBinary(input: Input) {
  binaryGuard(input);
  return byId(read, input.id);
}

export function afterBranches(input: Input) {
  branches(input);
  return byId(read, input.id);
}
