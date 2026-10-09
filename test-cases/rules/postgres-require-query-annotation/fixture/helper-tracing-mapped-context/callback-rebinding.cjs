// Rebinding a sloppy parameter also puts the callback in its mapped slot.
function forward(callback) {
  const query = sql``;
  var callback = () => query.append("SELECT 1");
  const ignored = unknownConsumer(arguments[0]);
  return query;
}
const result = forward("placeholder");
