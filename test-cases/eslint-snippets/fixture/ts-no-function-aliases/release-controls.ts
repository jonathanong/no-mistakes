import { imported, LIST, request } from "./api";
// Private receivers cannot replace their public facade for an outside caller.
const catalog = createCatalog();
export function isCode(v) { return catalog.isCode(v); }
const registry = createRegistry();
export function register(s) { return registry.register(s); }
const callbacks = [];
export function subscribe(cb) { return callbacks.push(cb); }
export function method(id) { return /^(POST|PUT):/.test(id); }
export function validate(s) { return /\bVALIDATE\b/i.test(s); }
const pattern = /value/;
export function matches(x) { return pattern.test(x); }
export function retrieve(id) { return getClient().refunds.retrieve(id); }
export function includes(v) { return (LIST as readonly string[]).includes(v); }
export function asserted(v) { return (request!).post(v); }
export function memberThis(v) { return this.handle(v); }
// Raw reachable identifiers below retain their released contract findings.
export function predicate(v: unknown): v is Code { return imported(v); }
export function narrowParameter(v: string) { return imported(v); }
export function narrowReturn(v): string { return imported(v); }
export function generic<T>(v: T) { return imported(v); }
export function reordered(first, second) { return imported(second, first); }
export function fixed(v) { return imported(v, "fixed"); }
export function changed(v) { return imported(normalize(v)); }
export function testClosure(slug, body) {
  const send = payload => request.post(`/x/${slug}`).send(payload);
  return send;
}
function privateFunction(v) { return v; }
export const privateForwarder = v => privateFunction(v);
export function nested(operation) { const local = v => operation(v); return local; }
const contextual: Narrow = v => imported(v);
let assigned: Narrow;
assigned = v => imported(v);
const object: { run: Narrow } = {};
object.run = v => imported(v);
export function shadow(imported) { return imported(imported); }

// Plain identifiers retain the released async/generator/statement findings.
export const asyncFacade = async value => await imported(value);
export function* generatorFacade(value) { return imported(value); }
export function discards(value) { imported(value); }
export function localRequired(value: string): string { return value; }
export const optionalFacade = (value?: string) => localRequired(value);
export const narrowedLocal = (value: number) => localRequired(value);
export const narrowerReturn = (value: string): never => localRequired(value);
export function overloaded(value: string): string;
export function overloaded(value: number): number;
export function overloaded(value: string | number) { return value; }
const overloadedFacade = (value: string) => overloaded(value);

// Plain identifier overloads retain released behavior, unlike expanded targets.
export function overloadedWrapper(value: unknown): unknown;
export function overloadedWrapper(value) { return imported(value); }
export function shadowedOverload(shadowedOverload: unknown): unknown;
export function shadowedOverload(shadowedOverload) { return imported(shadowedOverload); }
const missingReceiver = input => imported.call(input);
const wrongApplyList = input => imported.apply(input);
const assertedImport = input => imported!(input);
const assertedCall = input => imported!.call(undefined, input);
const assertedResult = input => imported(input)!;

import type { TypeOnly } from "./api";
import { type OtherType } from "./api";
const typeOnlyFacade = input => TypeOnly(input);
const otherTypeFacade = input => OtherType(input);
export type LocalType = string;
export type { LocalType as PublicType };
export { type LocalType as AnotherType };
export { remote } from "./api";
export * from "./api";
export { missing };
let mutable = value => value;
export { mutable };
const mutableFacade = value => mutable(value);
export const contextualTarget: Narrow = value => value;
const contextualTargetFacade = (value: string) => contextualTarget(value);
export function genericTarget<T>(value: T) { return value; }
const genericTargetFacade = (value: string) => genericTarget(value);
export function twoParameters(first: string, second?: number) { return first; }
const fewerParameters = (value: string) => twoParameters(value);

// Matching two opaque rest patterns is not forwarding one unchanged parameter.
const destructuredRest = (...[value]) => imported(...[value]);
const destructured = ({ value }) => imported(value);
const defaulted = (value = "default") => imported(value);

// Raw generic calls retain released behavior; asserted forwarding receivers stay excluded.
const explicitGeneric = value => imported<string>(value);
const explicitOptionalGeneric = value => imported?.<string>(value);
const explicitForwardGeneric = value => imported.call<string>(undefined, value);
import * as importedApi from "./api";
const assertedForwardReceiver = value => importedApi.create.call(importedApi!, value);
const assertedApplyReceiver = value => importedApi.create.apply(importedApi!, [value]);

// Default-expression exports snapshot the old function; a named live export would differ.
function snapshotOriginal(value) { return value; }
export default snapshotOriginal;
snapshotOriginal = replacement;
export const snapshotFacade = value => snapshotOriginal(value);

// Raw identifier optional/default contracts retain their released findings.
const optionalImported = (value?) => imported(value);
const optionalUntypedLocal = (value?): string => localRequired(value);
export function defaultedTarget(value: string = "default"): string { return value; }
const defaultContract = (value: string): string => defaultedTarget(value);
