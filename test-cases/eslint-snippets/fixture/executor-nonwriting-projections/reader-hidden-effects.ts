import sql from "@example/sql";
import { read, type SessionExecutor } from "@example/db";
import type { Input } from "./local";

function byId(query: SessionExecutor, id: string) {
  return query(sql`SELECT id FROM sample WHERE id = ${id}`);
}

const shared: Input = { id: "original", other: "data" };

function writesShared(input: Input) {
  shared.id = "changed";
  return input.id !== "";
}

function readsSharedGetter(input: Input) {
  return box.flag && input.id !== "";
}

const box = {
  get flag() {
    shared.id = "changed";
    return true;
  },
};

declare function opaqueEffect(): void;
function callsOpaqueEffect(input: Input) {
  opaqueEffect();
  return input.id !== "";
}

function bySharedWrite(input: Input) {
  writesShared(input);
  return byId(read, input.id);
}

function bySharedGetter(input: Input) {
  readsSharedGetter(input);
  return byId(read, input.id);
}

function byOpaqueEffect(input: Input) {
  callsOpaqueEffect(input);
  return byId(read, input.id);
}

export function runWithSharedInput() {
  return [bySharedWrite(shared), bySharedGetter(shared), byOpaqueEffect(shared)];
}

function pureReader(input: Input, minimum: number) {
  const hasId = input.id !== "";
  if (minimum > 0) return hasId && input.other !== "";
  return hasId;
}

export function acceptsLocalScalarLogic(input: Input) {
  pureReader(input, 1);
  return byId(read, input.id);
}

function checksEnabled(input: Input, enabled: boolean) {
  return enabled && input.id !== "";
}

function changesSharedInput() {
  shared.id = "changed";
  return true;
}

function byEffectfulArgument(input: Input) {
  checksEnabled(input, changesSharedInput());
  return byId(read, input.id);
}

function byGetterArgument(input: Input) {
  checksEnabled(input, box.flag);
  return byId(read, input.id);
}

export function runWithEffectfulArguments() {
  return [byEffectfulArgument(shared), byGetterArgument(shared)];
}
