export function target() {}
const original = { method: target, nested: { method: target } };
const aliases = { first: original.method, second: original.method };
aliases.first();
aliases.second();
original.nested.method();

// Reusing a self-linked object across completed projections is not recursion.
const linked = { self: linked, method: target };
linked.self.self.method();
