export function target() {}
const original = { method: target, nested: { method: target } };
const aliases = { first: original.method, second: original.method };
aliases.first();
aliases.second();
original.nested.method();

// Finite repeated projections through a self-linked object must stay concrete.
const linked = { self: linked, method: target };
linked.self.self.method();
