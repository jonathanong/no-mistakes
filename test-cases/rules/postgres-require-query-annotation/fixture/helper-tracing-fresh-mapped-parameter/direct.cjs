// The old container uncertainty does not undo this definite parameter assignment.
function forward(parameter) {
  const escaped = opaque(arguments);
  var parameter = '/* fresh annotation */ SELECT 1';
  return parameter;
}
const result = forward('placeholder');
