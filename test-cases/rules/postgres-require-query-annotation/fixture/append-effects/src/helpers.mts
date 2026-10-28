export function add(q, tail) { q.append(tail); }
export function addMore(q, tail) { q.append(tail).append(' '); }
export function returned(q, tail) { return q.append(tail); }
export function aliased(q, tail) { const alias = q.append(tail); opaque(alias); }
export function defaulted(q, other = opaque(q)) { q.append(other); }
export function captured(q, tail) { return () => q.append(tail); }
export function forwarded(q) { opaque(q); }
