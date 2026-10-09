// Arrow parameters are separate bindings even when arguments is inherited.
function forward(statement) {
  const arrow = (statement) => {
    var statement = "replacement";
    return arguments[0];
  };
  return arrow("SELECT 1");
}
const result = forward("/* lexical arguments */ SELECT 1");
