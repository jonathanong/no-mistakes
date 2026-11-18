import sql from "@example/sql";
import { read, type SessionExecutor } from "@example/db";
import type { Input } from "./local";

function byId(query: SessionExecutor, id: string) {
  return query(sql`SELECT id FROM sample WHERE id = ${id}`);
}

function primitiveUnions(input: { id: string | number; other: string | null }) {
  return input.id !== 0 && input.other !== null;
}

export function acceptsPrimitiveUnions(input: Input) {
  primitiveUnions(input);
  return byId(read, input.id);
}

function readAny(input: any) {
  return input.id;
}

function readUnknown(input: unknown) {
  return (input as Input).id;
}

function readEmpty(input: {}) {
  return (input as Input).id;
}

interface CallableInput extends Input {
  (): void;
}

function readCallable(input: CallableInput) {
  return input.id;
}

interface ConstructableInput extends Input {
  new (): object;
}

function readConstructable(input: ConstructableInput) {
  return input.id;
}

function readIndex(input: { id: string; [name: string]: string }) {
  return input.id;
}

function readOptional(input: { id: string; other?: string }) {
  return input.id;
}

function readObject(input: { id: string; other: unknown }) {
  return input.id;
}

function readGetter(input: { get id(): string; other: string }) {
  return input.id;
}

function defaultEscapes(input: Input, captured = input) {
  return captured.id;
}

function withRest(input: Input, ...rest: string[]) {
  return input.id + rest.join("");
}

function withOptional(input: Input, suffix?: string) {
  return input.id + suffix;
}

function withDestructured({ id }: Input) {
  return id;
}

function withArguments(input: Input) {
  return arguments[0].id;
}

function wrappedReceiver(input: Input) {
  return (input.id as unknown as () => boolean)();
}

function wrappedTag(input: Input) {
  return (input.id as unknown as (text: TemplateStringsArray) => boolean)`tag`;
}

async function asyncReader(input: Input) {
  return input.id;
}

function* generatorReader(input: Input) {
  yield input.id;
}

function optionalMember(input: Input) {
  return input?.id;
}

function noParameter() {
  return "";
}

declare const unionReader: ((input: Input) => string) | ((input: Input) => boolean);

function classCapture(input: Input) {
  return class {
    id = input.id;
  };
}

export function anyFormal(input: Input) {
  readAny(input);
  return byId(read, input.id);
}

export function unknownFormal(input: Input) {
  readUnknown(input);
  return byId(read, input.id);
}

export function emptyFormal(input: Input) {
  readEmpty(input);
  return byId(read, input.id);
}

export function callableFormal(input: CallableInput) {
  readCallable(input);
  return byId(read, input.id);
}

export function constructableFormal(input: ConstructableInput) {
  readConstructable(input);
  return byId(read, input.id);
}

export function indexedFormal(input: Input) {
  readIndex(input);
  return byId(read, input.id);
}

export function optionalProperty(input: Input) {
  readOptional(input);
  return byId(read, input.id);
}

export function nonprimitiveProperty(input: Input) {
  readObject(input);
  return byId(read, input.id);
}

export function getterProperty(input: Input) {
  readGetter(input);
  return byId(read, input.id);
}

export function defaultFormal(input: Input) {
  defaultEscapes(input);
  return byId(read, input.id);
}

export function restFormal(input: Input) {
  withRest(input);
  return byId(read, input.id);
}

export function optionalFormal(input: Input) {
  withOptional(input);
  return byId(read, input.id);
}

export function destructuredFormal(input: Input) {
  withDestructured(input);
  return byId(read, input.id);
}

export function argumentsRead(input: Input) {
  withArguments(input);
  return byId(read, input.id);
}

export function receiverCall(input: Input) {
  wrappedReceiver(input);
  return byId(read, input.id);
}

export function receiverTag(input: Input) {
  wrappedTag(input);
  return byId(read, input.id);
}

export function asyncCall(input: Input) {
  asyncReader(input);
  return byId(read, input.id);
}

export function generatorCall(input: Input) {
  generatorReader(input);
  return byId(read, input.id);
}

export function optionalPropertyAccess(input: Input) {
  optionalMember(input);
  return byId(read, input.id);
}

export function missingFormal(input: Input) {
  noParameter(input);
  return byId(read, input.id);
}

export function unionCallee(input: Input) {
  unionReader(input);
  return byId(read, input.id);
}

export function nestedClass(input: Input) {
  classCapture(input);
  return byId(read, input.id);
}

export function acceptsCallableProjection(input: CallableInput) {
  return byId(read, input.id);
}

export function acceptsConstructableProjection(input: ConstructableInput) {
  return byId(read, input.id);
}
