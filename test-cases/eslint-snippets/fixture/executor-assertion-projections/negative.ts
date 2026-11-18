import { read } from "@example/db";
import { captureInAssertion, replaced, writeInAssertion } from "./helpers";
import { byId, type Input } from "./query";

declare function ensure(value: unknown, detail?: unknown): asserts value;
declare function booleanCheck(value: unknown): boolean;
declare function assertionPredicate(value: unknown): value is Input;
declare function otherPosition(first: unknown, second: unknown): asserts second;
declare function mutate(value: Input): void;
declare function getStatus(): number;
declare const unchecked: any;
declare const unknownCheck: unknown;
declare const unionCheck: ((value: unknown) => asserts value) | ((value: Input) => asserts value);

function overloaded(value: unknown): asserts value;
function overloaded(value: unknown, code: number): asserts value;
function overloaded(value: unknown, code?: number): asserts value {
  (value as Input).id = "changed";
}

declare function rebound(value: unknown): asserts value;
rebound = writeInAssertion;

export function afterUnknownMutation(input: Input) {
  ensure(input);
  mutate(input);
  return byId(read, input.id);
}

export function afterFieldWrite(input: Input) {
  ensure(input);
  input.id = "changed";
  return byId(read, input.id);
}

export function afterAssertionWrite(input: Input) {
  writeInAssertion(input);
  return byId(read, input.id);
}

export function afterCapturedAssertion(input: Input) {
  captureInAssertion(input);
  return byId(read, input.id);
}

export function afterAny(input: Input) {
  unchecked(input);
  return byId(read, input.id);
}

export function afterUnknown(input: Input) {
  unknownCheck(input);
  return byId(read, input.id);
}

export function afterBoolean(input: Input) {
  booleanCheck(input);
  return byId(read, input.id);
}

export function afterTypeGuard(input: Input) {
  assertionPredicate(input);
  return byId(read, input.id);
}

export function afterShadow(input: Input, ensure: (value: unknown) => asserts value) {
  ensure(input);
  return byId(read, input.id);
}

export function afterReassignment(input: Input) {
  rebound(input);
  return byId(read, input.id);
}

export function afterImportedReassignment(input: Input) {
  replaced(input);
  return byId(read, input.id);
}

export function withWrongAssertionParameter(input: Input) {
  otherPosition(input, true);
  return byId(read, input.id);
}

export function withRepeatedArgument(input: Input) {
  ensure(input, input);
  return byId(read, input.id);
}

export function withEffectfulArgument(input: Input) {
  ensure(input, getStatus());
  return byId(read, input.id);
}

export function withSpread(input: Input) {
  ensure(...([input] as const));
  return byId(read, input.id);
}

export function afterUnion(input: Input) {
  unionCheck(input);
  return byId(read, input.id);
}

export function afterOverload(input: Input) {
  overloaded(input);
  return byId(read, input.id);
}
