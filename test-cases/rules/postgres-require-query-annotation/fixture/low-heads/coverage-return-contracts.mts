import { read } from "@bench/db";
import sql from "sql-template-strings";
function literal(flag) { if (flag) return "/* one */ SELECT 1"; return "/* two */ SELECT 2"; }
function wrapped(flag) { if (flag) { return `/* one */ SELECT 1`; } else { return `/* two */ SELECT 2`; } return "/* fallback */ SELECT 3"; }
function mixedBuilder(flag) { if (flag) return sql`/* one */ SELECT 1`; return "/* two */ SELECT 2"; }
function missing(flag) { if (flag) return "SELECT 1"; return "SELECT 2"; }
function mixedProof(flag) { if (flag) return "/* one */ SELECT 1"; return "SELECT 2"; }
function nestedReturn(flag) { if (flag) { if (other) return "/* one */ SELECT 1"; } return "/* two */ SELECT 2"; }
function valueless(flag) { if (flag) return; return "/* two */ SELECT 2"; }
function throwOnly() { throw Error("bad"); }
function returnsUnknown() { return runtimeValue; }
function abruptGood(value) { switch (value) { case "a": return "/* one */ SELECT 1"; default: return throwOnly(); } }
function abruptUnknown(value) { switch (value) { case "a": return "/* one */ SELECT 1"; default: return returnsUnknown(); } }
function switchThrow(value) { switch (value) { case "a": throw Error("bad"); case "b": return "/* one */ SELECT 1"; default: throw Error("bad"); } }
function switchDefaultValue(value) { switch (value) { case "a": return "/* one */ SELECT 1"; default: return "/* two */ SELECT 2"; } }
function switchExtraStatement(value) { switch (value) { case "a": opaque(); return "/* one */ SELECT 1"; default: throw Error("bad"); } }
function switchNoDefault(value) { switch (value) { case "a": return "/* one */ SELECT 1"; } }
function generatorNested(flag) { function* inner() { yield 1; } if (flag) return "/* one */ SELECT 1"; return "/* two */ SELECT 2"; }
function stableLocal(flag) { const query = sql`/* local */ SELECT 1`; if (flag) return query; return query; }
function changedLocal(flag) { let query = sql`/* local */ SELECT 1`; query = unknownValue; if (flag) return query; return query; }
function capturedLocal(flag) { const query = sql`/* local */ SELECT 1`; function nested() { return query; } if (flag) return query; return nested(); }
function mutatedBuilder(flag) { const query = sql`/* local */ SELECT 1`; opaque(query); if (flag) return query; return query; }
read(literal(true)); // low: true
read(wrapped(true)); // low: true
read(mixedBuilder(true)); // low: true
read(missing(true)); // low: false
read(mixedProof(true)); // low: false
read(nestedReturn(true)); // low: false
read(valueless(true)); // low: false
read(abruptGood("a")); // low: true
read(abruptUnknown("a")); // low: false
read(switchThrow("b")); // low: true
read(switchDefaultValue("a")); // low: false
read(switchExtraStatement("a")); // low: false
read(switchNoDefault("a")); // low: false
read(generatorNested(true)); // low: true
read(stableLocal(true)); // low: true
read(changedLocal(true)); // low: false
read(capturedLocal(true)); // low: false
read(mutatedBuilder(true)); // low: false
