// Opaque mapped reads preserve callbacks inside promises and aggregates.
function forward(parameter) {
  const query = sql``;
  const escaped = opaque(arguments);
  var parameter = [Promise.resolve(() => query.append('SELECT 1'))];
  const escapedAgain = opaque(arguments);
  const invoked = opaque(parameter);
  return query;
}
const result = forward('placeholder');
