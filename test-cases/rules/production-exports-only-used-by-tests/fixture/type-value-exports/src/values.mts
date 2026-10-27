// Value exports: a type-position reference is not a runtime production use.
export function fnImportType() { return 1; }
export const constInline = 2;
export function fnQualified() { return 3; }
export function fnImportQualified() { return 4; }
export function fnNamespace() { return 5; }
export function fnChain() { return 6; }
export let letValue = 7;
export var varValue = 8;
// Type-like exports: a type reference keeps them consumed.
export class KlassType { id = 1; }
export interface IFace { id: number }
export enum Status { On, Off }
