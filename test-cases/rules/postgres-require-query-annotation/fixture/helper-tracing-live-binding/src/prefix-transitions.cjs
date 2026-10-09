// Saved prefixes represent the states compared when an alias becomes uncertain.
const annotated = sql`/* annotation */ SELECT 1`;
const bare = sql`SELECT 1`;

// Sparse module slots may replace this callback on a later alternative arm.
const callback = () => annotated;
