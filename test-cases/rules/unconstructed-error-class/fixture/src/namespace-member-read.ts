import * as lib from "./namespace-member-read-lib";

// A member read through the namespace import is a use of that export alone:
// `version` is no namespace, so `ReadKept` is still reported; `ReadHanded` is
// handed on, so it stays quiet.
export const read = lib.version;
export const handed = lib.ReadHanded;
