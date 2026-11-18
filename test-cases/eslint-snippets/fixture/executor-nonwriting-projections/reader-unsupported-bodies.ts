import sql from "@example/sql";
import { read, type SessionExecutor } from "@example/db";
import type { Input } from "./local";

function byId(query: SessionExecutor, id: string) {
  return query(sql`SELECT id FROM sample WHERE id = ${id}`);
}

function mutableLocal(input: Input) {
  let key: string;
  key = input.id;
  return key !== "";
}

function restBinding(input: Input) {
  const { id, ...rest } = input;
  return id !== "" && rest.other !== "";
}

declare function resolveKey(): string;
function defaultBinding(input: Input) {
  const { id = resolveKey() } = input;
  return id !== "";
}

declare function keyName(): "id";
function computedBinding(input: Input) {
  const { [keyName()]: id } = input;
  return id !== "";
}

function nestedBinding(input: Input) {
  const { id: { length } } = input;
  return length > 0;
}

declare const external: Input;
function externalBinding(input: Input) {
  const { id } = external;
  return input.id === id;
}

function loopReader(input: Input) {
  for (const character of input.id) {
    if (character === "x") return true;
  }
  return false;
}

export function projectionAfterMutableLocal(input: Input) {
  mutableLocal(input);
  return byId(read, input.id);
}

export function projectionAfterRestBinding(input: Input) {
  restBinding(input);
  return byId(read, input.id);
}

export function projectionAfterDefaultBinding(input: Input) {
  defaultBinding(input);
  return byId(read, input.id);
}

export function projectionAfterComputedBinding(input: Input) {
  computedBinding(input);
  return byId(read, input.id);
}

export function projectionAfterNestedBinding(input: Input) {
  nestedBinding(input);
  return byId(read, input.id);
}

export function projectionAfterExternalBinding(input: Input) {
  externalBinding(input);
  return byId(read, input.id);
}

export function projectionAfterLoop(input: Input) {
  loopReader(input);
  return byId(read, input.id);
}

function readsId(input: Input) {
  return input.id !== "";
}
const aliasedReader = readsId;
export function rejectsAliasedCallee(input: Input) {
  aliasedReader(input);
  return byId(read, input.id);
}

function readsExtra(input: Input & { extra: string }) {
  return input.extra !== "";
}
export function rejectsAssertionArgument(input: Input) {
  readsExtra(input as Input & { extra: string });
  return byId(read, input.id);
}
