import DefaultValue, { value as alias, type Shape, onlyType } from "./external";
import type TypeDefault from "./types";
import * as namespace from "./external";
import "./side-effect";
export { alias as renamed };
export { value as forwarded, type Shape as ForwardedShape } from "./external";
export * from "./star";
export * as forwardedNamespace from "./namespace";
export type * from "./types";
export const local = DefaultValue(alias);
export default local;
type Description = Shape & TypeDefault & typeof onlyType;
const typed: onlyType = namespace.value;
// The parameter must resolve locally rather than count as a use of imported alias.
function shadow(alias: number) { return alias; }
{ const alias = 1; alias; }
const deferred = import("./literal");
const required = require("./required");
