function forward(statement) {
  const read = () => arguments[0];
  const ignored = (arguments = ["SELECT 1"]);
  return read();
}
const result = forward("/* captured arguments replacement */ SELECT 1");
