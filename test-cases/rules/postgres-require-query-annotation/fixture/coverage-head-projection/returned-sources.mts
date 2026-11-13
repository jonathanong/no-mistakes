import sql from "sql-template-strings";
export function literal(q) { const result = q.append(" FROM items"); return result; }
export function trusted(q) { const result = q.append(sql` FROM items`); return result; }
export function untrusted(q) { const result = q.append(unknownTag` FROM items`); return result; }
export function interpolated(q) { const result = q.append(sql` FROM ${table}`); return result; }
export function template(q) { const result = q.append(` FROM items`); return result; }
export function noAppend(q) { const result = q; return result; }
export function wrongReturn(q) { const result = q.append(" FROM items"); return q; }
export function mutableAlias(q) { let result = q.append(" FROM items"); return result; }
export function optional(q) { const result = q.append?.(" FROM items"); return result; }
export function wrongBase(q) { const result = other.append(" FROM items"); return result; }
export function extraEffect(q) { const result = q.append(" FROM items"); opaque(result); return result; }
export function eight(q) { const result = q.append(" ").append(" ").append(" ").append(" ").append(" ").append(" ").append(" ").append(" "); return result; }
export function nine(q) { const result = q.append(" ").append(" ").append(" ").append(" ").append(" ").append(" ").append(" ").append(" ").append(" "); return result; }
