// Sloppy simple parameters share their argument slots; reassignment revokes proof.
function forward(statement) {
  const before = arguments[0];
  var statement = "SELECT 1";
  return arguments[0];
}
const result = forward("/* initial annotation */ SELECT 1");
