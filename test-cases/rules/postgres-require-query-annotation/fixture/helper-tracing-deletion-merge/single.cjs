// A slot deleted on one branch may still expose its mapped parameter.
function forward(parameter) {
  const ignored = condition ? delete arguments[0] : false;
  const escaped = opaque(arguments);
  return parameter;
}
const result = forward('/* possible deletion */ SELECT 1');
