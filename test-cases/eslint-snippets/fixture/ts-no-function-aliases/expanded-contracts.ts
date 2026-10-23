import * as api from "./api";
import { original } from "./api";
// Expanded callee/assignment/cast forms do not receive identifier compatibility.
const typedMember = (value: string): string => api.create(value);
const typedForward = (value: string) => original.call(undefined, value);
const asyncMember = async value => api.create(value);
const statementMember = value => { api.create(value); };
const contextualMember: F = value => api.create(value);
const predicateMember = (value: unknown): value is Code => api.create(value);
function* generatorMember(value) { return api.create(value); }
/** @param {Code} value @returns {boolean} */
const documentedMember = value => api.create(value);
const aliases = {};
aliases[`template`] = value => original(value);
const assertedTarget = value => original!(value);
const assertedResult = value => original(value)!;
async function awaitedAssertedResult(value) { return await original(value)!; }
const instantiatedForward = value => original.call<string>(undefined, value);
// A declared expanded wrapper must prove parameter annotation presence too.
export function exportedMixed(first: string, second: number): boolean { return Boolean(first && second); }
const missingAnnotation = (first: string, second): boolean => exportedMixed.call(undefined, first, second);
// Fully matching primitive declarations remain proven forwarding aliases.
export function primitive(value: string): boolean { return Boolean(value); }
const proven = (value: string): boolean => primitive.call(undefined, value);
export function untyped(value): boolean { return Boolean(value); }
const provenUntyped = (value): boolean => untyped.call(undefined, value);
export function required(value: string): string { return value; }
const optional = (value?: string) => required.call(undefined, value);
const mismatched = (value: number) => required.call(undefined, value);
const narrowerReturn = (value: string): never => required.call(undefined, value);
const wrapperGeneric = <T>(value: T) => required.call(undefined, value);
export function genericTarget<T>(value: T) { return value; }
const targetGeneric = (value: string) => genericTarget.call(undefined, value);
export const annotatedTarget: F = value => value;
const targetContext = (value: string) => annotatedTarget.call(undefined, value);
export function moreParams(first: string, second?: number): string { return first; }
const arity = (value: string) => moreParams.call(undefined, value);
export function defaultTarget(value: string = "default"): string { return value; }
const targetDefault = (value: string) => defaultTarget.call(undefined, value);
const discarded = (value: string) => { required.call(undefined, value); };
const predicate = (value: string): value is Code => required.call(undefined, value);
export function optionalTarget(value?: string): string { return value ?? ""; }
const optionalMissingAnnotation = (value?): string => optionalTarget.call(undefined, value);
const optionalProven = (value?: string): string => optionalTarget.call(undefined, value);
export function voidTarget(value: string): void { audit(value); }
const voidProven = (value: string): void => { voidTarget.call(undefined, value); };
export function overloadedMember(value: string): string;
export function overloadedMember(value) { return api.create(value); }
const explicitMemberGeneric = value => api.create<string>(value);
/** A description alone leaves this expanded contract unchanged. */
const documented = value => api.create(value);
export function overloadedTarget(value: string): string;
export function overloadedTarget(value: number): number;
export function overloadedTarget(value: string | number) { return value; }
const overloadedForward = value => overloadedTarget.call(undefined, value);
