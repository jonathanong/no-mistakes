import * as target from "./member-read-target";

// A member read through the namespace import is a use of that export alone:
// `version` is no namespace, `ReadHanded` is one that is handed on.
export const read = target.version;
export const handed = target.ReadHanded;
