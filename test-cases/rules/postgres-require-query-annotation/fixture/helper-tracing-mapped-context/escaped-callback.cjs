// Escaped invocation objects still carry subsequently assigned callbacks.
function forward(callback) {
  const query = sql``;
  const escaped = unknownConsumer(arguments);
  var callback = () => query.append("SELECT 1");
  const ignored = unknownConsumer(arguments[0]);
  return query;
}
const result = forward("placeholder");
