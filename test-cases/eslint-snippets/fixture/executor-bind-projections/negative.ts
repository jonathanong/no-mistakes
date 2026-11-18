import sql, { type SqlStatement } from "@example/sql";
import { read, type SessionExecutor } from "@example/db";
import { fixedProjection, typedScalar, type Input } from "./local";

export function unknownField(query: SessionExecutor, input: { id: unknown }) {
  const { id } = input;
  return query(sql`SELECT id FROM sample WHERE id = ${id}`);
}
export function anyField(query: SessionExecutor, input: { id: any }) {
  return query(sql`SELECT id FROM sample WHERE id = ${input.id}`);
}
export function statementField(query: SessionExecutor, input: { id: SqlStatement }) {
  return query(sql`SELECT id FROM sample WHERE id = ${input.id}`);
}
export function optionalField(query: SessionExecutor, input: { id?: string }) {
  return query(sql`SELECT id FROM sample WHERE id = ${input.id}`);
}
export function mixedField(query: SessionExecutor, input: { id: string | SqlStatement }) {
  return query(sql`SELECT id FROM sample WHERE id = ${input.id}`);
}
export function missingField(query: SessionExecutor, input: object) {
  const { id } = input as Input;
  return query(sql`SELECT id FROM sample WHERE id = ${id}`);
}
export function fieldAsStatement(query: SessionExecutor, input: { statement: string }) {
  return query(input.statement);
}
export function concatenated(query: SessionExecutor, fragment: string) {
  return query("SELECT id FROM sample WHERE " + fragment);
}
export function rawTemplate(query: SessionExecutor, input: Input) {
  return query(`SELECT id FROM sample WHERE id = '${input.id}'`);
}
export function localTag(strings: TemplateStringsArray, ...values: unknown[]) {
  return "SELECT 1";
}
export function untrusted(query: SessionExecutor, input: Input) {
  return query(localTag`SELECT id FROM sample WHERE id = ${input.id}`);
}
export function changedBinding(query: SessionExecutor, input: Input) {
  let { id } = input;
  id = "different";
  return query(sql`SELECT id FROM sample WHERE id = ${id}`);
}
export function changedObject(query: SessionExecutor, input: Input) {
  input.id = "different";
  return query(sql`SELECT id FROM sample WHERE id = ${input.id}`);
}
declare function mutate(input: Input): void;
export function escapedObject(query: SessionExecutor, input: Input) {
  mutate(input);
  return query(sql`SELECT id FROM sample WHERE id = ${input.id}`);
}
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
export function opaqueMember(query: SessionExecutor, input: { id: unknown }) {
  return typedScalar(query, input.id);
}
export function castMember(query: SessionExecutor, input: unknown) {
  return typedScalar(query, (input as Input).id);
}
export function dynamicMember(query: SessionExecutor, input: Input, key: keyof Input) {
  return typedScalar(query, input[key]);
}
export function nestedMember(query: SessionExecutor, input: { nested: Input }) {
  return typedScalar(query, input.nested.id);
}
export function optionalMember(query: SessionExecutor, input: Input | undefined) {
  return typedScalar(query, input?.id!);
}
export function accessorMember(query: SessionExecutor, input: { get id(): string }) {
  return typedScalar(query, input.id);
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

export function opaqueUnion(input: Input | { id: SqlStatement }) {
  return anyField(read, input);
}
export function reassignedCaller(input: Input) {
  input = { id: "updated" };
  return fixedProjection(read, input);
}
export function escapedCaller(input: Input) {
  mutate(input);
  return fixedProjection(read, input);
}
export function aliasedCaller(input: Input) {
  const alias = input;
  return fixedProjection(read, input);
}
