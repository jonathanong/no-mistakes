import { "value" as named } from "./external";
export { named as "quoted" };
export function functionValue() { return named; }
export class ClassValue {}
export interface InterfaceValue {}
export type AliasValue = string;
export enum EnumValue { Member }
declare const ambient: string;
export { ambient };
type ExternalType = import("./external").Shape;
