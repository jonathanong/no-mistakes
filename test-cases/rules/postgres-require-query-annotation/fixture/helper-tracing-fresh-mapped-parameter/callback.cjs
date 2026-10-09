// A definite rebinding remains observable after an earlier arguments escape.
function forward(callback) {
  const query = sql``;
  const escaped = opaque(arguments);
  var callback = () => query.append('SELECT 1');
  const invoked = opaque(callback);
  return query;
}
const result = forward('placeholder');
