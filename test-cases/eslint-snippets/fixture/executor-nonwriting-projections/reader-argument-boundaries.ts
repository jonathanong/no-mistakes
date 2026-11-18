import sql from "@example/sql";
import { read, type SessionExecutor } from "@example/db";
import { importedProjectionEnabled } from "@example/flags";
import type { Input } from "./local";

function byId(query: SessionExecutor, id: string) {
  return query(sql`SELECT id FROM sample WHERE id = ${id}`);
}

function checksEnabled(input: Readonly<Input>, enabled: boolean) {
  return enabled && input.id !== "";
}

function checksPrimitiveForms(
  input: Readonly<Input>,
  score: number,
  unused: undefined,
  type: string,
) {
  return input.id !== "" && score < 0 && type === "number";
}

export function primitiveOperators(input: Input) {
  checksPrimitiveForms(input, -1, void 0, typeof 1);
  return byId(read, input.id);
}

export function overBudgetAlias(input: Input) {
  const a01 = true;
  const a02 = a01;
  const a03 = a02;
  const a04 = a03;
  const a05 = a04;
  const a06 = a05;
  const a07 = a06;
  const a08 = a07;
  const a09 = a08;
  checksEnabled(input, a09);
  return byId(read, input.id);
}

export function reassignedParameter(input: Input, enabled: boolean) {
  enabled = !enabled;
  checksEnabled(input, enabled);
  return byId(read, input.id);
}

export function mutableLocal(input: Input) {
  let enabled: boolean;
  enabled = true;
  checksEnabled(input, enabled);
  return byId(read, input.id);
}

export function mutableBinding(input: Input) {
  let enabled = true;
  checksEnabled(input, enabled);
  return byId(read, input.id);
}

export function ambientArgument(input: Input) {
  checksEnabled(input, ambientProjectionEnabled);
  return byId(read, input.id);
}

export function importedArgument(input: Input) {
  checksEnabled(input, importedProjectionEnabled);
  return byId(read, input.id);
}

declare const flags: { get enabled(): boolean };
export function getterInitializer(input: Input) {
  const enabled = flags.enabled;
  checksEnabled(input, enabled);
  return byId(read, input.id);
}
