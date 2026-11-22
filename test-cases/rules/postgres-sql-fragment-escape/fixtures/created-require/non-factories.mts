import "node:module";
import { createRequire, isBuiltin } from "node:module";
import { type createRequire as inlineType } from "node:module";
import type { createRequire as wholeType } from "module";

const make = createRequire;
const sources = { builtins: { createRequire } };

let mutable = createRequire(import.meta.url);
const aliased = make(import.meta.url);
const inline = inlineType(import.meta.url);
const erased = wholeType(import.meta.url);
const unrelated = isBuiltin(import.meta.url);
const nested = sources.builtins.createRequire(import.meta.url);

mutable("../mutable.cjs");
aliased("../aliased.cjs");
inline("../inline-type.cjs");
erased("../whole-type.cjs");
unrelated("../unrelated.cjs");
nested("../nested.cjs");

const direct = createRequire(import.meta.url);
direct("../direct.cjs");
