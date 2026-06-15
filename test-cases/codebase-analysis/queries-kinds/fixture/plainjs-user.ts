// `./plain.js` has no TS source, so it resolves to the checked-in `.js`;
// `./ghost.js` has neither a source nor a literal file, so it is unresolved.
import { p } from "./plain.js";
import "./ghost.js";

export const q = p;
