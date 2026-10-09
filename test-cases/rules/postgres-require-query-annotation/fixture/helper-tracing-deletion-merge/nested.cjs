// The nested alternative has one surviving slot path.
function forward(parameter) {
  const ignored = condition ? (other ? delete arguments[0] : false) : delete arguments[0];
  const escaped = opaque(arguments);
  return parameter;
}
const result = forward('/* nested possible deletion */ SELECT 1');
