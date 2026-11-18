import { read } from "@example/db";
import { byId, type Input } from "./query";

declare function inspect(value: unknown): boolean;
declare function errorValue(): Error;
declare function errorMessage(): string;
declare const constructors: { Error: ErrorConstructor };

function effectfulGuard(value: unknown): asserts value {
  if (inspect(value)) throw "Missing";
}

function propertyGuard(value: Input): asserts value {
  if (value.id) throw "Missing";
}

function unsupportedBinary(value: unknown): asserts value {
  if (value == null) throw "Missing";
}

function negatedCall(value: unknown): asserts value {
  if (!inspect(value)) throw "Missing";
}

function unknownThrow(value: unknown): asserts value {
  if (!value) throw errorValue();
}

function otherConstructor(value: unknown): asserts value {
  if (!value) throw new TypeError("Missing");
}

function memberConstructor(value: unknown): asserts value {
  if (!value) throw new constructors.Error("Missing");
}

function dynamicMessage(value: unknown): asserts value {
  if (!value) throw new Error(errorMessage());
}

function defaultParameter(value: unknown = {}): asserts value {
  if (!value) throw "Missing";
}

function restParameter(value: unknown, ...rest: unknown[]): asserts value {
  if (!value) throw "Missing";
}

function destructuredParameter(value: unknown, { enabled }: { enabled: boolean }): asserts value {
  if (!value) throw "Missing";
}

export function afterEffectfulGuard(input: Input) {
  effectfulGuard(input);
  return byId(read, input.id);
}

export function afterPropertyGuard(input: Input) {
  propertyGuard(input);
  return byId(read, input.id);
}

export function afterUnsupportedBinary(input: Input) {
  unsupportedBinary(input);
  return byId(read, input.id);
}

export function afterNegatedCall(input: Input) {
  negatedCall(input);
  return byId(read, input.id);
}

export function afterUnknownThrow(input: Input) {
  unknownThrow(input);
  return byId(read, input.id);
}

export function afterOtherConstructor(input: Input) {
  otherConstructor(input);
  return byId(read, input.id);
}

export function afterMemberConstructor(input: Input) {
  memberConstructor(input);
  return byId(read, input.id);
}

export function afterDynamicMessage(input: Input) {
  dynamicMessage(input);
  return byId(read, input.id);
}

export function afterDefaultParameter(input: Input) {
  defaultParameter(input);
  return byId(read, input.id);
}

export function afterRestParameter(input: Input) {
  restParameter(input);
  return byId(read, input.id);
}

export function afterDestructuredParameter(input: Input) {
  destructuredParameter(input, true as unknown as { enabled: boolean });
  return byId(read, input.id);
}

function unsupportedUnary(value: unknown, status: number): asserts value {
  if (-status) throw "Missing";
}

export function afterUnsupportedUnary(input: Input) {
  unsupportedUnary(input, 404);
  return byId(read, input.id);
}
