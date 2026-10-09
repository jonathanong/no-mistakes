function forward() {
  const query = sql``;
  // Sparse conditional callbacks must survive sidecar slot joins and traversal.
  const assigned = flag ? (arguments[1000] = () => query.append('SELECT 1')) : false;
  const consumed = opaque(arguments);
  return query;
}
const result = forward();
