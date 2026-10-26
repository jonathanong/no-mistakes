import { original } from "./api";
// These five direct identifier shapes were findings in the actual v0.95.0 rule.
export function annotated(value: string): string { return original(value); }
export const typedArrow = (value: Foo) => original(value);
export async function asynchronous(value) { return original(value); }
export function discarded(value) { original(value); }
export const contextual: F = value => original(value);
export async function awaited(value) { return await original(value); }
export function predicate(value: unknown): value is Code { return original(value); }
export function* generator(value) { return original(value); }
export function generic<T>(value: T) { return original(value); }
const optional = (value?) => original(value);
export function overloaded(value: string): string;
export function overloaded(value) { return original(value); }
let assigned: F;
assigned = value => original(value);
const object: { run: F } = {};
object.run = value => original(value);
export function exportedOverload(value: string): string;
export function exportedOverload(value: number): number;
export function exportedOverload(value: string | number) { return value; }
const overloadedTarget = value => exportedOverload(value);
