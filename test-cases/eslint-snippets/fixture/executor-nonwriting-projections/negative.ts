import sql from "@example/sql";
import { read, type SessionExecutor } from "@example/db";
import type { Input } from "./local";

function byId(query: SessionExecutor, id: string) {
  return query(sql`SELECT id FROM sample WHERE id = ${id}`);
}

function writeField(input: Input) {
  input.id = "changed";
}

function bypassReadonly(input: Readonly<Input>) {
  (input as Input).id = "changed";
}

function aliasWhole(input: Input) {
  const alias = input;
  return alias.id;
}

let stored: Input;
function storeWhole(input: Input) {
  stored = input;
}

function returnWhole(input: Input) {
  return input;
}

function captureWhole(input: Input) {
  return () => input.id;
}

declare function ambientMutable(input: Input): boolean;
declare function ambientReadonly(input: Readonly<Input>): boolean;
declare const anyConsumer: any;
declare function unknownConsumer(input: unknown): unknown;
function canInspect(input: Input) {
  return input.id !== "";
}

function overloaded(input: Input): boolean;
function overloaded(input: Input, enabled: boolean): boolean;
function overloaded(input: Input, enabled = true) {
  return enabled && input.id !== "";
}

function reassigned(input: Input) {
  return input.id;
}
reassigned = returnWhole as unknown as typeof reassigned;

export function mutated(input: Input) {
  writeField(input);
  return byId(read, input.id);
}

export function mutatedReadonly(input: Input) {
  bypassReadonly(input);
  return byId(read, input.id);
}

export function aliased(input: Input) {
  aliasWhole(input);
  return byId(read, input.id);
}

export function storedObject(input: Input) {
  storeWhole(input);
  return byId(read, input.id);
}

export function returnedObject(input: Input) {
  returnWhole(input);
  return byId(read, input.id);
}

export function captured(input: Input) {
  captureWhole(input);
  return byId(read, input.id);
}

export function ambientMutableCall(input: Input) {
  ambientMutable(input);
  return byId(read, input.id);
}

export function ambientReadonlyCall(input: Input) {
  ambientReadonly(input);
  return byId(read, input.id);
}

export function anyBefore(input: Input) {
  anyConsumer(input);
  return byId(read, input.id);
}

export function unknownBefore(input: Input) {
  unknownConsumer(input);
  return byId(read, input.id);
}

export function anyAfter(input: Input) {
  const { id } = input;
  const rows = byId(read, id);
  anyConsumer(input);
  return rows;
}

export function unknownAfter(input: Input) {
  const { id } = input;
  const rows = byId(read, id);
  unknownConsumer(input);
  return rows;
}

export function reassignedField(input: Input) {
  const { id } = input;
  input.id = "changed";
  return byId(read, id);
}

export function spreadCall(input: Input) {
  canInspect(...([input] as const));
  return byId(read, input.id);
}

export function optionalCall(input: Input) {
  canInspect?.(input);
  return byId(read, input.id);
}

export function overloadedCall(input: Input) {
  overloaded(input);
  return byId(read, input.id);
}

export function reassignedCall(input: Input) {
  reassigned(input);
  return byId(read, input.id);
}
