// Value exports: a type-position reference is not a runtime production use.
export function fnImportType() { return 1; }
export const constInline = 2;
export function fnQualified() { return 3; }
export function fnImportQualified() { return 4; }
export function fnNamespace() { return 5; }
export function fnChain() { return 6; }
export let letValue = 7;
export var varValue = 8;
// An enum is a runtime object: a type reference is not a production use. Status
// is used at runtime only by tests and typed in production, so it is reported.
export enum Status { On, Off }
// A const enum behaves like Status: typed in production, read at runtime by tests.
export const enum ConstStatus { On = 1, Off = 2 }
// Typed everywhere and never read at runtime: no consumer, so not reported.
export enum TypeOnlyLevel { Low = "low" }
// Type-like exports: a type reference keeps them consumed.
export class KlassType { id = 1; }
export interface IFace { id: number }
