function forward() {
  let statement = sql`/* surviving alias */ SELECT 1`;
  const alias = statement;
  // Replacing one reference never mutates the builder held by the other alias.
  const ignored = flag ? (statement = 'replacement') : false;
  return alias;
}
const result = forward();
