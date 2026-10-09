// An opaque update on just one branch still prevents proving the joined value.
function forward(parameter) {
  const ignored = condition ? (opaque(arguments), delete arguments[0]) : delete arguments[0];
  return parameter;
}
const result = forward('/* stale one-arm parameter */ SELECT 1');
