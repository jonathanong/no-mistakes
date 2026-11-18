import { read, write, type SessionExecutor } from "@example/db";
import {
  unionProjection,
  fixedProjection,
  renamedProjection,
  parameterProjection,
  memberProjection,
  primitiveFields,
  typedScalar,
  type Input,
  type Fields,
} from "./local";

export function withObject(input: Input) {
  return fixedProjection(write, input);
}
export function withLiteral() {
  return fixedProjection(write, { id: "sample" });
}
export function withRename(input: Input) {
  return renamedProjection(read, input);
}
export function withParameter(input: Input) {
  return parameterProjection(read, input);
}
export function withMember(input: Input) {
  return memberProjection(read, input);
}
export function withFields(input: Fields) {
  return primitiveFields(read, input);
}
export function scalarMember(query: SessionExecutor, input: Input) {
  return typedScalar(query, input.id);
}
export function scalarParameter(query: SessionExecutor, id: string) {
  return typedScalar(query, id);
}

export function withUnion(input: Input | { id: number }) {
  return unionProjection(read, input);
}
