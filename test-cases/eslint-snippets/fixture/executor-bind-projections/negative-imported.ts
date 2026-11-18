import { read, type SessionExecutor } from "@example/db";
import { fixedProjection, typedScalar, type Input } from "./local";
import {
  unknownField,
  anyField,
  statementField,
  optionalField,
  mixedField,
  missingField,
  fieldAsStatement,
  concatenated,
  rawTemplate,
  untrusted,
  changedBinding,
  changedObject,
  escapedObject,
} from "./negative";
import type { SqlStatement } from "@example/sql";

export function unknownInput(input: { id: unknown }) {
  return unknownField(read, input);
}
export function anyInput(input: { id: any }) {
  return anyField(read, input);
}
export function statementInput(input: { id: SqlStatement }) {
  return statementField(read, input);
}
export function optionalInput(input: { id?: string }) {
  return optionalField(read, input);
}
export function mixedInput(input: { id: string | SqlStatement }) {
  return mixedField(read, input);
}
export function missingInput(input: object) {
  return missingField(read, input);
}
export function sourceStatement(input: { statement: string }) {
  return fieldAsStatement(read, input);
}
export function sourceFragment(fragment: string) {
  return concatenated(read, fragment);
}
export function sourceTemplate(input: Input) {
  return rawTemplate(read, input);
}
export function sourceUntrusted(input: Input) {
  return untrusted(read, input);
}
export function bindingWrite(input: Input) {
  return changedBinding(read, input);
}
export function propertyWrite(input: Input) {
  return changedObject(read, input);
}
export function objectEscape(input: Input) {
  return escapedObject(read, input);
}
export function callbackControl() {
  return fixedProjection((statement) => read(statement), { id: "sample" });
}
