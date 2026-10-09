// Named default declarations create module bindings.
export default function sql() { return sql`/* self */ SELECT 1`; }
// Redeclaring the named default function mutates its module binding.
var sql = otherTag;
