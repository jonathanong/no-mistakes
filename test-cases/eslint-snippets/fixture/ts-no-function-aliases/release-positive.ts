import original, { create, batch } from "./api";
import * as api from "./api";
export const direct = input => create(input);
const defaultForward = input => original(input);
const member = input => api.client.create(input);
const stringMember = input => api["create"](input);
const templateMember = input => api[`create`](input);
const optionalMember = input => api?.create(input);
const optionalCall = input => api.create?.(input);
const call = input => create.call(undefined, input);
const apply = input => create.apply(undefined, [input]);
const methodCall = input => api.client.create.call(api.client, input);
const methodApply = input => api.client.create.apply(api.client, [input]);
const rest = (...values) => batch(...values);
const restApply = (...values) => batch.apply(undefined, [...values]);
export function local(value: string): boolean { return value.length > 0; }
export const typed = (value: string): boolean => local(value);
export const localConst = value => value;
const localForward = value => localConst(value);
function listed(value) { return value; }
export { listed };
const listedForward = value => listed(value);
// Reachable recursion still adds no alias to another implementation.
export function recursive(value) { return recursive(value); }
export function recursiveCall(value) { return recursiveCall.call(undefined, value); }
export const recursiveApply = value => recursiveApply.apply(undefined, [value]);

export function voidTarget(value: string): void { audit(value); }
const voidForwarder = (value: string): void => { voidTarget(value); };
export function localPlain(value: number): number { return value; }
function localFunctionForwarder(value: number): number { return localPlain(value); }

const actualCallExport = input => api.call(input);
const actualApplyExport = input => api.apply(input);

export function mixed(first: string, second: number): boolean { return Boolean(first && second); }
const mixedForwarder = (first: string, second): boolean => mixed(first, second);
function defaultCallable(value) { return value; }
export default defaultCallable;
const defaultIdentifierForwarder = value => defaultCallable(value);
const parenthesizedForwarder = value => (api.create?.call)(api, value);
