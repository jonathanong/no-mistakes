// Every retained container alias must see a mapped parameter rebinding.
async function promise(value) {
  return value;
}
function forward(statement) {
  const nested = [arguments];
  const pending = promise(arguments);
  const unrelated = other("SELECT 2");
  var statement = "SELECT 1";
  return arguments[0];
}
const result = forward("/* annotated */ SELECT 1");
function opaqueValue(statement) {
  var statement = unknownValue;
  return arguments[0];
}
const unknownResult = opaqueValue("/* annotated */ SELECT 1");
function disconnected(statement) {
  const ignored = delete arguments[0];
  var statement = "SELECT 2";
  return arguments[0];
}
const deletedResult = disconnected("/* annotated */ SELECT 1");
function shadows(statement) {
  const local = () => {
    const statement = "SELECT 1";
    return statement;
  };
  const hoisted = () => {
    function statement() {
      return "SELECT 1";
    }
    return statement();
  };
  const bare = () => {
    var statement;
    return statement;
  };
  const a = local();
  const b = hoisted();
  const c = bare();
  return statement;
}
const shadowResult = shadows("/* preserved */ SELECT 1");

// Unsupported destructuring can escape lexical arguments, so fail closed.
function reservedShadow(statement) {
  const reserved = () => {
    const { statement } = unknownObject;
    return statement;
  };
  const ignored = reserved();
  return statement;
}
const reservedResult = reservedShadow("/* maybe invalidated */ SELECT 1");
