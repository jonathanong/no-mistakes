import * as lib from "./namespace-member-read-lib";

// A member read through the namespace import is a use of that export alone:
// `version` is no namespace, so `ReadKept` is still reported; `ReadHanded` is
// handed on, so it stays quiet. A string literal names the member the same way
// a dot does.
export const read = lib.version;
export const handed = lib.ReadHanded;
export const readByString = lib["version"];
export const handedByString = lib["ReadLiteral"];
