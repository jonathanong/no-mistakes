function forward(statement) {
  const ignored = (arguments = ["SELECT 1"]);
  return arguments[0];
}
const result = forward("/* mapped arguments replacement */ SELECT 1");
