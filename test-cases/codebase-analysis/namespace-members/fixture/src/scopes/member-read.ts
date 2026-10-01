import * as target from "./member-read-target";

// A member read through the namespace import is a use of that export alone:
// `version` is no namespace, `ReadHanded` is one that is handed on. A string
// literal names the member the same way a dot does.
export const read = target.version;
export const handed = target.ReadHanded;
export const readByString = target["version"];
export const handedByString = target["ReadLiteral"];
