const database = require("@app/db");
const builder = sql`/* builder snapshot */ SELECT 1`;
builder.append(" WHERE TRUE");
const promise = async () => builder;
const promised = promise();
function create(parameter) {
  const cycle = (arguments[1000] = arguments);
  var parameter = "/* fresh snapshot */ SELECT 1";
  return () => [parameter, arguments, builder, promised];
}
const observer = create("original");
function createStateful(parameter) {
  return () => {
    const saved = arguments[0];
    const assigned = (arguments[0] = "SELECT 2");
    database.query(saved);
  };
}
const first = createStateful("/* first */ SELECT 1");
const second = createStateful("/* second */ SELECT 1");

// Memo invalidation classifies summary effects, not uninvoked helper creation.
function mutationHoisted() {
  function local() {
    return builder;
  }
  return local;
}
function mutationAppend() {
  builder.append(" WHERE extra");
}
function mutationTemplate() {
  return `prefix ${opaque(builder)}`;
}
function mutationSequence() {
  return (opaque(builder), "done");
}
